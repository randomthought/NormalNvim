use crate::models::{
    order::{FilledOrder, OrderId, OrderResult, PendingOrder},
    security::Ticker,
};
use anyhow::{bail, Result};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct Orders {
    filled_orders: RwLock<HashMap<OrderId, FilledOrder>>,
    pending_orders: RwLock<HashMap<Ticker, HashMap<OrderId, PendingOrder>>>,
}

impl Orders {
    pub fn new() -> Self {
        Self {
            filled_orders: RwLock::new(HashMap::new()),
            pending_orders: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_orders(&self) -> Vec<OrderResult> {
        todo!()
    }

    pub async fn get_order(&self, order_id: OrderId) -> Option<OrderResult> {
        todo!()
    }

    pub async fn insert(&self, order_result: &OrderResult) {
        match order_result {
            OrderResult::FilledOrder(o) => self.insert_filled(o).await,
            OrderResult::PendingOrder(o) => self.insert_pending(o).await,
        }
    }

    pub async fn remove(&self, order_id: &OrderId) -> Result<()> {
        let mut map = self.pending_orders.write().await;
        let Some(_) = map.get(order_id) else {
            bail!("order not found");
        };

        map.remove(order_id);

        Ok(())
    }

    async fn insert_filled(&self, pending_order: &FilledOrder) {
        // TODO: handle a market order that closes another
        // TODO: handle market order that partially reduces/expands another

        let order_id = &pending_order.order_id;
        let mut map = self.filled_orders.write().await;
        map.insert(order_id.to_owned(), pending_order.to_owned());
    }

    async fn insert_pending(&self, pending_order: &PendingOrder) {
        let order_id = &pending_order.order_id;
        let security = pending_order.order.get_security();

        let mut map = self.pending_orders.write().await;
        if let Some(os) = map.get_mut(&security.ticker) {
            os.insert(order_id.to_owned(), pending_order.to_owned());

            return;
        }

        let mut orders = HashMap::new();
        orders.insert(order_id.to_owned(), pending_order.to_owned());
        map.insert(security.ticker.to_owned(), orders);
    }
}
