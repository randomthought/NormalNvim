use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::models::{
    order::{FilledOrder, OrderDetails, OrderId, OrderResult},
    security::{Security, Ticker},
};
use anyhow::Result;

use super::active_order::ActiveOrder;

pub struct Orders {
    active: RwLock<HashMap<Ticker, ActiveOrder>>,
}

impl Orders {
    pub fn new() -> Self {
        Self {
            active: RwLock::new(HashMap::new()),
        }
    }
    pub async fn insert(&self, order_result: &OrderResult) {
        match order_result {
            OrderResult::FilledOrder(_) => todo!(),
            OrderResult::PendingOrder(_) => todo!(),
        }
    }

    pub async fn get_orders(&self) -> Vec<OrderResult> {
        todo!()
    }

    pub async fn get_order(&self, security: &Security) -> Option<FilledOrder> {
        todo!()
    }

    pub async fn remove(&self, order_id: &OrderId) -> Result<()> {
        todo!()
    }

    async fn handle_filled(&self, filled_order: &FilledOrder) {
        todo!()
    }
}

fn to_order_details(filled_order: &FilledOrder) -> OrderDetails {
    todo!()
}
