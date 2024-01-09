use crate::{
    data::QouteProvider,
    event::{
        self,
        event::{EventHandler, EventProducer},
        model::Event,
    },
    models::{
        order::{self, FilledOrder, Order, OrderResult, PendingOrder},
        price::{Price, Quote},
        security::Security,
    },
    order::{Account, OrderManager, OrderReader},
};
use anyhow::{bail, ensure, Ok, Result};
use async_trait::async_trait;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, ops::Mul, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::orders::Orders;

pub struct Broker {
    event_producer: Arc<dyn EventProducer + Sync + Send>,
    qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
    // TODO: leveage needs to be float for example 1.5 leverage
    leverage: f64,
    account_balance: RwLock<Decimal>,
    orders: Orders,
    commissions_per_share: Decimal,
}

impl Broker {
    pub fn new(
        account_balance: Decimal,
        qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
        event_producer: Arc<dyn EventProducer + Sync + Send>,
    ) -> Self {
        let commissions_per_share = Decimal::from_f64(0.0).unwrap();
        Self {
            leverage: 10.0,
            event_producer,
            account_balance: RwLock::new(account_balance),
            commissions_per_share,
            orders: Orders::new(),
            qoute_provider,
        }
    }

    async fn create_trade(&self, market_order: &order::Market) -> Result<(Price, FilledOrder)> {
        let quote = self
            .qoute_provider
            .get_quote(&market_order.security)
            .await?;

        let price = match market_order.side {
            order::Side::Long => quote.bid,
            order::Side::Short => quote.ask,
        };
        let Some(active) = self.orders.get_order(&market_order.security).await else {
            let cost = Decimal::from_u64(market_order.quantity).unwrap() * price;
            let filled_order = create_filled_order(market_order.quantity, &market_order.security, market_order.side, &quote)?;
            return Ok((cost, filled_order));
        };

        // TODO: what if quantity are equal and side are different

        if active.side == market_order.side {
            let cost = Decimal::from_u64(market_order.quantity).unwrap() * price;
            let filled_order = create_filled_order(
                market_order.quantity,
                &market_order.security,
                market_order.side,
                &quote,
            )?;
            return Ok((cost, filled_order));
        }

        if active.quantity == market_order.quantity {
            let cost = Decimal::from_u64(0).unwrap();
            let filled_order = create_filled_order(
                market_order.quantity,
                &market_order.security,
                market_order.side,
                &quote,
            )?;
            return Ok((cost, filled_order));
        }

        let (quantity, side) = get_new_order_specs(&active, market_order)?;
        let cost = Decimal::from_u64(market_order.quantity).unwrap() * price;
        let filled_order = create_filled_order(quantity, &market_order.security, side, &quote)?;

        return Ok((cost, filled_order));
    }

    async fn process_order(&self, order: &Order) -> Result<()> {
        let e = match self.place_order(order).await? {
            OrderResult::FilledOrder(o) => Event::FilledOrder(o),
            OrderResult::PendingOrder(o) => Event::OrderTicket(o),
        };

        self.event_producer.produce(e).await?;

        Ok(())
    }
}

#[async_trait]
impl Account for Broker {
    async fn get_account_balance(&self) -> Result<Decimal> {
        let balance = self.account_balance.read().await;
        Ok(*balance)
    }
    async fn get_buying_power(&self) -> Result<Decimal> {
        let balance = self.account_balance.read().await;
        let l = Decimal::from_f64(self.leverage).unwrap();

        Ok(balance.mul(l))
    }
}

#[async_trait]
impl OrderReader for Broker {
    async fn open_orders(&self) -> Result<Vec<OrderResult>> {
        let orders = self.orders.get_orders().await;
        let results: Vec<_> = orders
            .iter()
            .filter(|o| matches!(o, OrderResult::FilledOrder(_)))
            .collect();

        Ok(orders)
    }

    async fn pending_orders(&self) -> Result<Vec<OrderResult>> {
        let orders = self.orders.get_orders().await;
        let results: Vec<_> = orders
            .iter()
            .filter(|o| matches!(o, OrderResult::PendingOrder(_)))
            .collect();

        Ok(orders)
    }
}

#[async_trait]
impl OrderManager for Broker {
    async fn place_order(&self, order: &Order) -> Result<OrderResult> {
        if let Order::StopLimitMarket(o) = order {
            let m = order::Market::new(o.quantity, o.side, o.security.clone(), o.times_in_force);

            let market_order = Order::Market(m);
            self.place_order(&market_order).await?;
            // TODO: handle limit orders
        }

        let Order::Market(market_order) = order else {
            let po = order::PendingOrder {
                order_id: Uuid::new_v4().to_string(),
                order: order.clone(),
            };

            let or = order::OrderResult::PendingOrder(po.clone());

            self.orders.insert(&or).await;

            return Ok(or);
        };

        // TODO: check if you can afford the trade first

        let mut account_balance = self.account_balance.write().await;

        let (cost, filled_order) = self.create_trade(market_order).await?;

        if cost >= *account_balance {
            bail!("do not have enough funds to peform trade");
        }

        let order_result = order::OrderResult::FilledOrder(filled_order.clone());
        self.orders.insert(&order_result);
        let commision =
            Decimal::from_u64(market_order.quantity).unwrap() * self.commissions_per_share;
        *account_balance = *account_balance - (commision + cost);

        Ok(order_result)
    }

    async fn update(&self, pending_order: &PendingOrder) -> Result<()> {
        let or = order::OrderResult::PendingOrder(pending_order.to_owned());
        self.orders.insert(&or).await;

        Ok(())
    }

    async fn cancel(&self, pending_order: &PendingOrder) -> Result<()> {
        self.orders.remove(&pending_order).await
    }
}

#[async_trait]
impl EventHandler for Broker {
    async fn handle(&self, event: &Event) -> Result<()> {
        if let Event::Order(o) = event {
            self.process_order(o).await?;
            return Ok(());
        }

        let Event::Market(event::model::Market::DataEvent(d)) = event else {
          return Ok(())
        };

        let Some(candle) = d.history.last() else {
          return Ok(())
        };

        let security = &d.security;
        let pending = self.orders.get_pending_orders(security).await;

        for p in pending {
            match p.order {
                Order::Limit(o) => {
                    let met = match o.side {
                        order::Side::Long => o.price >= candle.close,
                        order::Side::Short => o.price <= candle.close,
                    };
                    if !met {
                        continue;
                    }

                    // TODO: with this implementation, you would not get the exact limit price
                    let m = order::Market::new(o.quantity, o.side, o.security, o.times_in_force);
                    let order = Order::Market(m);
                    self.place_order(&order).await?;
                }
                Order::StopLimitMarket(o) => todo!(),
                _ => continue,
            };
        }

        todo!()
    }
}

fn create_filled_order(
    quantity: u64,
    security: &Security,
    side: order::Side,
    quote: &Quote,
) -> Result<FilledOrder> {
    let price = match side {
        order::Side::Long => quote.ask,
        order::Side::Short => quote.bid,
    };

    let order_id = Uuid::new_v4().to_string();
    let fo = FilledOrder {
        price,
        side,
        quantity,
        order_id: order_id.clone(),
        security: security.clone(),
        datetime: SystemTime::now().duration_since(UNIX_EPOCH)?,
    };

    Ok(fo)
}

fn get_new_order_specs(
    active_order: &FilledOrder,
    market_order: &order::Market,
) -> Result<(u64, order::Side)> {
    ensure!(
        active_order.quantity != market_order.quantity,
        "quantities cannot be equal"
    );
    ensure!(
        active_order.side != market_order.side,
        "side cannot be equal"
    );

    if active_order.quantity > market_order.quantity {
        let quantity = active_order.quantity - market_order.quantity;
        return Ok((quantity, active_order.side));
    }

    let quantity = market_order.quantity - active_order.quantity;
    return Ok((quantity, market_order.side));
}
