use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::models::{
    order::{FilledOrder, OrderDetails, OrderId, OrderResult, PendingOrder},
    security::Security,
};
use anyhow::{bail, Ok, Result};

use super::active_order::ActiveOrder;

pub struct Orders {
    active: RwLock<HashMap<Security, ActiveOrder>>,
    pendig: RwLock<HashMap<Security, HashMap<OrderId, PendingOrder>>>,
}

impl Orders {
    pub fn new() -> Self {
        Self {
            pendig: RwLock::new(HashMap::new()),
            active: RwLock::new(HashMap::new()),
        }
    }

    pub async fn insert(&self, order_result: &OrderResult) {
        match order_result {
            OrderResult::FilledOrder(o) => self.handle_filled(o).await,
            OrderResult::PendingOrder(_) => todo!(),
        }
    }

    pub async fn get_orders(&self) -> Vec<OrderResult> {
        todo!()
    }

    pub async fn get_pending_orders(&self, security: &Security) -> Vec<PendingOrder> {
        let map = self.pendig.read().await;
        let Some(pds) = map.get(security) else {
            return vec![];
        };
        let v: Vec<_> = pds.values().map(|p| p.to_owned()).collect();
        v
    }

    pub async fn get_order(&self, security: &Security) -> Option<FilledOrder> {
        let map = self.active.read().await;
        let Some(active) = map.get(security) else {
            return None;
        };

        let Some(position) = active.get_position() else {
            return None;
        };

        let filled_order = FilledOrder {
            order_id: todo!(),
            price: todo!(),
            datetime: todo!(),
            security: security.to_owned(),
            side: position.side,
            quantity: position.quantity,
        };

        Some(filled_order)
    }

    pub async fn remove(&self, pending_order: &PendingOrder) -> Result<()> {
        let security = pending_order.order.get_security();
        let mut map = self.pendig.write().await;
        let Some(security_orders) = map.get_mut(security) else {
            bail!("order doesn't exist");
        };

        let oder_id = &pending_order.order_id;
        let Some(_) = security_orders.remove(oder_id) else {
            bail!("order doesn't exist");
        };

        Ok(())
    }

    async fn handle_filled(&self, filled_order: &FilledOrder) {
        let mut map = self.active.write().await;
        let order_details = to_order_details(filled_order);
        if let Some(active_order) = map.get_mut(&filled_order.security) {
            active_order.insert(order_details);
            return;
        }

        let mut active_order = ActiveOrder::new(filled_order.security.to_owned());
        active_order.insert(order_details);
        map.insert(filled_order.security.to_owned(), active_order);
    }
}

fn to_order_details(filled_order: &FilledOrder) -> OrderDetails {
    OrderDetails {
        datetime: filled_order.datetime.to_owned(),
        order_id: filled_order.order_id.to_owned(),
        price: filled_order.price.to_owned(),
        quantity: filled_order.quantity,
        side: filled_order.side,
    }
}
