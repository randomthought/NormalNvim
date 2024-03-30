use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::models::{
    orders::{
        filled_order::FilledOrder, new_order::NewOrder, order_result::OrderResult,
        pending_order::PendingOrder, security_position::SecurityPosition,
    },
    security::Security,
};

use super::{
    pending::{Pending, PendingKey},
    security_transaction::SecurityTransaction,
};

pub struct Orders {
    active: RwLock<HashMap<Security, SecurityTransaction>>,
    pending: RwLock<Pending>,
}

impl Orders {
    pub fn new() -> Self {
        Self {
            active: RwLock::new(HashMap::new()),
            pending: RwLock::new(Pending::default()),
        }
    }

    pub async fn get_transactions(&self) -> Result<Vec<SecurityTransaction>, String> {
        let map = self.active.read().await;
        let transactions = map.iter().map(|kv| kv.1.to_owned()).collect();
        Ok(transactions)
    }

    pub async fn insert(&self, order_result: &OrderResult) -> Result<(), String> {
        match order_result {
            OrderResult::FilledOrder(o) => self.handle_filled(o).await,
            OrderResult::PendingOrder(o) => self.handle_pending(o).await,
            _ => todo!("return error with unsupoorted order type"),
        }
    }

    pub async fn get_position(&self, security: &Security) -> Option<SecurityPosition> {
        let map = self.active.read().await;
        let Some(active) = map.get(security) else {
            return None;
        };

        active.get_position()
    }

    pub async fn get_positions(&self) -> Vec<SecurityPosition> {
        let active_map = self.active.read().await;

        active_map
            .values()
            .flat_map(|ao| ao.get_position())
            .collect()
    }

    pub async fn get_pending_orders(&self) -> Vec<PendingOrder> {
        let pending = self.pending.read().await;
        pending.get(PendingKey::None)
    }

    pub async fn get_pending_order(&self, pending_key: PendingKey) -> Vec<PendingOrder> {
        let pending = self.pending.read().await;
        pending.get(pending_key)
    }

    pub async fn remove(&self, pending_order: &PendingOrder) -> Result<(), String> {
        let key = PendingKey::OrderIdKey(pending_order.order_id.to_owned());
        let mut pending = self.pending.write().await;
        pending.remove(key);
        Ok(())
    }

    async fn handle_filled(&self, filled_order: &FilledOrder) -> Result<(), String> {
        let mut map = self.active.write().await;
        if let Some(active_order) = map.get_mut(&filled_order.security) {
            active_order.insert(filled_order)?;
            return Ok(());
        }

        let mut active_order = SecurityTransaction::new(filled_order.security.to_owned());
        active_order.insert(filled_order)?;
        map.insert(filled_order.security.to_owned(), active_order);
        return Ok(());
    }

    async fn handle_pending(&self, pending_order: &PendingOrder) -> Result<(), String> {
        if let NewOrder::Market(_) = pending_order.order.to_owned() {
            return Err("market orders should immidiatly be executed".into());
        }

        let mut pending = self.pending.write().await;
        pending.update(pending_order.to_owned());

        Ok(())
    }
}
