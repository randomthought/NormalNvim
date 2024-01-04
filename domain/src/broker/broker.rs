use crate::{
    data::QouteProvider,
    event::{
        event::{EventHandler, EventProducer},
        model::Event,
    },
    models::{
        order::{self, FilledOrder, Order, OrderResult, PendingOrder},
        security::Security,
    },
    order::{Account, OrderManager, OrderReader},
};
use anyhow::{bail, Ok, Result};
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
    leverage: u32,
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
            leverage: 10,
            event_producer,
            account_balance: RwLock::new(account_balance),
            commissions_per_share,
            orders: Orders::new(),
            qoute_provider,
        }
    }

    async fn create_filled_order(
        &self,
        quantity: u64,
        security: &Security,
        side: order::Side,
    ) -> Result<FilledOrder> {
        let quote = self.qoute_provider.get_quote(security).await?;
        let price = match side {
            order::Side::Long => quote.ask,
            order::Side::Short => quote.bid,
        };
        let q = Decimal::from_u64(quantity).unwrap();
        let total_commision = self.commissions_per_share * q;

        let order_id = Uuid::new_v4().to_string();
        let fo = FilledOrder {
            price,
            side,
            quantity,
            order_id: order_id.clone(),
            security: security.clone(),
            commission: total_commision,
            datetime: SystemTime::now().duration_since(UNIX_EPOCH)?,
        };

        Ok(fo)
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
        let l = Decimal::from_u32(self.leverage).unwrap();

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
        }

        let Order::Market(o) = order else {
            let po = order::PendingOrder {
                order_id: Uuid::new_v4().to_string(),
                order: order.clone(),
            };

            let or = order::OrderResult::PendingOrder(po.clone());

            self.orders.insert(&or).await;

            return Ok(or);
        };

        let filled_order = self
            .create_filled_order(o.quantity, &o.security, o.side)
            .await?;

        let q = Decimal::from_u64(filled_order.quantity).unwrap();
        let cost = filled_order.commission + (q * filled_order.price);

        let mut current_balance = self.account_balance.write().await;
        if cost > *current_balance {
            bail!("not enough funds to perform the trade");
        }

        *current_balance = *current_balance - cost;

        let or = order::OrderResult::FilledOrder(filled_order.clone());
        self.orders.insert(&or).await;

        return Ok(or);
    }

    async fn update(&self, pending_order: &PendingOrder) -> Result<()> {
        let or = order::OrderResult::PendingOrder(pending_order.to_owned());
        self.orders.insert(&or).await;

        Ok(())
    }

    async fn cancel(&self, pending_order: &PendingOrder) -> Result<()> {
        self.orders.remove(&pending_order.order_id).await
    }
}

#[async_trait]
impl EventHandler for Broker {
    async fn handle(&self, event: &Event) -> Result<()> {
        if let Event::Order(o) = event {
            self.process_order(o).await?
        }

        let Event::Market(m) = event else {
          return Ok(())
        };

        Ok(())
    }
}
