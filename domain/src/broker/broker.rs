use crate::{
    data::QouteProvider,
    event::{
        self,
        event::{EventHandler, EventProducer},
        model::{AlgoOrder, Event},
    },
    models::{
        order::{self, FilledOrder, NewOrder, OrderResult, PendingOrder, SecurityPosition},
        price::{Price, Quote},
        security::Security,
    },
    order::{Account, OrderManager, OrderReader},
};
use async_trait::async_trait;
use color_eyre::eyre::{bail, Ok, Result};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::orders::Orders;

pub struct Broker {
    event_producer: Arc<dyn EventProducer + Sync + Send>,
    qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
    // TODO: leveage needs to be float for example 1.5 leverage
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

        let price = match market_order.order_details.side {
            order::Side::Long => quote.bid,
            order::Side::Short => quote.ask,
        };
        let Some(active) = self.orders.get_position(&market_order.security).await else {
            let cost = Decimal::from_u64(market_order.order_details.quantity).unwrap() * -price;
            let filled_order = create_filled_order(
                market_order.order_details.quantity,
                &market_order.security,
                market_order.order_details.side,
                &quote,
            )?;
            return Ok((cost, filled_order));
        };

        if active.side == market_order.order_details.side {
            let filled_order = create_filled_order(
                market_order.order_details.quantity,
                &market_order.security,
                market_order.order_details.side,
                &quote,
            )?;
            let cost = calculate_cost(&active, &filled_order);
            return Ok((cost, filled_order));
        }

        let active_position_quantity = active.get_quantity();
        if active_position_quantity == market_order.order_details.quantity {
            let filled_order = create_filled_order(
                market_order.order_details.quantity,
                &market_order.security,
                market_order.order_details.side,
                &quote,
            )?;

            let cost = calculate_cost(&active, &filled_order);
            return Ok((cost, filled_order));
        }

        let side = if active_position_quantity > market_order.order_details.quantity {
            active.side
        } else {
            market_order.order_details.side
        };

        let filled_order = create_filled_order(
            market_order.order_details.quantity,
            &market_order.security,
            side,
            &quote,
        )?;

        let cost = calculate_cost(&active, &filled_order);
        return Ok((cost, filled_order));
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
        Ok(*balance)
    }
}

#[async_trait]
impl OrderReader for Broker {
    async fn get_positions(&self) -> Result<Vec<SecurityPosition>> {
        let orders = self.orders.get_positions().await;
        Ok(orders)
    }

    async fn get_pending_orders(&self) -> Result<Vec<OrderResult>> {
        let orders = self.orders.get_pending_orders().await;
        let order_results = orders
            .iter()
            .map(|p| OrderResult::PendingOrder(p.to_owned()))
            .collect();

        Ok(order_results)
    }
}

#[async_trait]
impl OrderManager for Broker {
    async fn place_order(&self, order: &NewOrder) -> Result<OrderResult> {
        if let NewOrder::StopLimitMarket(o) = order {
            let market_order = NewOrder::Market(o.market.to_owned());
            self.place_order(&market_order).await?;

            let oca = NewOrder::OCA(o.one_cancels_other.to_owned());
            return self.place_order(&oca).await;
        }

        let NewOrder::Market(market_order) = order else {
            let po = order::PendingOrder {
                order_id: Uuid::new_v4().to_string(),
                order: order.clone(),
            };

            let or = order::OrderResult::PendingOrder(po.clone());

            self.orders.insert(&or).await?;

            return Ok(or);
        };

        let mut account_balance = self.account_balance.write().await;

        let (cost, filled_order) = self.create_trade(market_order).await?;

        if (cost + *account_balance) < Decimal::new(0, 0) {
            bail!("do not have enough funds to peform trade");
        }

        let order_result = order::OrderResult::FilledOrder(filled_order.clone());
        self.orders.insert(&order_result).await?;
        let commision = Decimal::from_u64(market_order.order_details.quantity).unwrap()
            * self.commissions_per_share;
        let trade_cost = commision + cost;
        *account_balance += trade_cost;

        Ok(order_result)
    }

    async fn update(&self, pending_order: &PendingOrder) -> Result<OrderResult> {
        let or = order::OrderResult::PendingOrder(pending_order.to_owned());
        self.orders.insert(&or).await?;

        Ok(OrderResult::Updated(pending_order.order_id.to_owned()))
    }

    async fn cancel(&self, pending_order: &PendingOrder) -> Result<OrderResult> {
        self.orders.remove(&pending_order).await?;
        Ok(OrderResult::Cancelled(pending_order.order_id.to_owned()))
    }
}

#[async_trait]
impl EventHandler for Broker {
    async fn handle(&self, event: &Event) -> Result<()> {
        let Event::Market(event::model::Market::DataEvent(d)) = event else {
            return Ok(());
        };

        let Some(candle) = d.history.last() else {
            return Ok(());
        };

        let security = &d.security;
        let pending = self.orders.get_pending_order(security).await;

        for p in pending {
            match p.order {
                NewOrder::Limit(o) => {
                    let met = match o.order_details.side {
                        order::Side::Long => o.price >= candle.close,
                        order::Side::Short => o.price <= candle.close,
                    };
                    if !met {
                        continue;
                    }

                    // TODO: with this implementation, you would not get the exact limit price
                    let m = order::Market::new(
                        o.order_details.quantity,
                        o.order_details.side,
                        o.security,
                    );
                    let order = NewOrder::Market(m);
                    self.place_order(&order).await?;
                }
                NewOrder::StopLimitMarket(o) => todo!(),
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

    let datetime = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let fo = FilledOrder::new(
        security.to_owned(),
        order_id,
        price,
        quantity,
        side,
        datetime,
    );

    Ok(fo)
}

fn calculate_cost(security_position: &SecurityPosition, filled_order: &FilledOrder) -> Price {
    let quantity = if security_position.side == filled_order.order_details.side {
        -Decimal::from_u64(filled_order.order_details.quantity).unwrap()
    } else {
        Decimal::from_u64(filled_order.order_details.quantity).unwrap()
    };

    quantity * filled_order.price
}
