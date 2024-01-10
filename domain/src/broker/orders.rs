use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::models::{
    order::{FilledOrder, OrderDetails, OrderId, OrderResult, PendingOrder, SecurityPosition},
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

    pub async fn insert(&self, order_result: &OrderResult) -> Result<()> {
        match order_result {
            OrderResult::FilledOrder(o) => self.handle_filled(o).await,
            OrderResult::PendingOrder(_) => todo!(),
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

    pub async fn get_pending_orders(&self, security: &Security) -> Vec<PendingOrder> {
        let map = self.pendig.read().await;
        let Some(pds) = map.get(security) else {
            return vec![];
        };

        pds.values().map(|p| p.to_owned()).collect()
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

    async fn handle_filled(&self, filled_order: &FilledOrder) -> Result<()> {
        let mut map = self.active.write().await;
        if let Some(active_order) = map.get_mut(&filled_order.security) {
            active_order.insert(filled_order)?;
            return Ok(());
        }

        let mut active_order = ActiveOrder::new(filled_order.security.to_owned());
        active_order.insert(filled_order)?;
        map.insert(filled_order.security.to_owned(), active_order);
        return Ok(());
    }
}

fn to_order_results(active_order: &ActiveOrder) -> OrderResult {
    todo!()
}
