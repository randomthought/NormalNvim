use crate::{
    data::QouteProvider,
    event::{
        event::{EventHandler, EventProducer},
        model::Event,
    },
    models::{
        order::{self, FilledOrder, Order, OrderId, OrderResult, PendingOrder},
        price::{Quote, Symbol},
        security::{self, Security},
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

pub struct Broker {
    event_producer: Arc<dyn EventProducer + Sync + Send>,
    qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
    leverage: u32,
    account_balance: RwLock<Decimal>,
    filled_orders: RwLock<HashMap<OrderId, FilledOrder>>,
    pending_orders: RwLock<HashMap<OrderId, PendingOrder>>,
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
            filled_orders: RwLock::new(HashMap::new()),
            pending_orders: RwLock::new(HashMap::new()),
            qoute_provider,
        }
    }

    async fn create_filled_order(
        &self,
        quantity: u32,
        security: &Security,
        side: order::Side,
    ) -> Result<FilledOrder> {
        let quote = self.qoute_provider.get_quote(security).await?;
        let price = match side {
            order::Side::Long => quote.ask,
            order::Side::Short => quote.bid,
        };
        let q = Decimal::from_u32(quantity).unwrap();
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
}

#[async_trait]
impl Account for Broker {
    async fn get_account_balance(&self) -> Result<Decimal> {
        let balance = self.account_balance.read().await;
        todo!()
    }
    async fn get_buying_power(&self) -> Result<Decimal> {
        let balance = self.account_balance.read().await;
        let l = Decimal::from_u32(self.leverage).unwrap();

        Ok(balance.mul(l))
    }
}

#[async_trait]
impl OrderReader for Broker {
    async fn orders(&self) -> Result<Vec<OrderResult>> {
        let orders = self.filled_orders.read().await;
        let results: Vec<_> = orders.values().map(|o| o.clone()).collect();

        todo!()
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

            let mut map = self.pending_orders.write().await;
            map.insert(po.order_id.clone(), po.clone());

            return Ok(or);
        };

        let filled_order = self
            .create_filled_order(o.quantity, &o.security, o.side)
            .await?;

        let q = Decimal::from_u32(filled_order.quantity).unwrap();
        let cost = filled_order.commission + (q * filled_order.price);

        let mut current_balance = self.account_balance.write().await;
        if cost > *current_balance {
            bail!("not enough funds to perform the trade");
        }

        let mut map = self.filled_orders.write().await;
        map.insert(filled_order.order_id.clone(), filled_order.clone());

        let or = order::OrderResult::FilledOrder(filled_order.clone());

        *current_balance = *current_balance - cost;

        return Ok(or);
    }

    async fn update(&self, pending_order: &PendingOrder) -> Result<()> {
        let mut map = self.pending_orders.write().await;
        let Some(_) = map.get(&pending_order.order_id) else {
            // TODO: what about limit market orders
            bail!("can only update limit orders");
        };

        let p = PendingOrder {
            order_id: pending_order.order_id.clone(),
            order: pending_order.order.clone(),
        };
        map.insert(pending_order.order_id.clone(), p);

        Ok(())
    }

    async fn cancel(&self, pending_order: &PendingOrder) -> Result<()> {
        let mut map = self.pending_orders.write().await;
        let Some(_) = map.get(&pending_order.order_id) else {
            // TODO: what about limit market orders
            bail!("can only cancel limit orders");
        };

        map.remove(&pending_order.order_id);

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Broker {
    async fn handle(&self, event: &Event) -> Result<()> {
        if let Event::Order(o) = event {
            let e = match self.place_order(o).await? {
                OrderResult::FilledOrder(o) => Event::FilledOrder(o),
                OrderResult::PendingOrder(o) => Event::OrderTicket(o),
            };

            self.event_producer.produce(e).await?
        }

        Ok(())
    }
}
