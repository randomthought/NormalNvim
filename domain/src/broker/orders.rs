use std::{collections::HashMap, error::Error};

use futures_util::future::ok;
use tokio::sync::RwLock;

use crate::models::{
    order::{FilledOrder, Limit, NewOrder, OrderId, OrderResult, PendingOrder, SecurityPosition},
    security::Security,
};
use color_eyre::eyre::{bail, Ok, Result};

use super::security_transaction::SecurityTransaction;

pub struct Orders {
    active: RwLock<HashMap<Security, SecurityTransaction>>,
    pending: RwLock<HashMap<Security, HashMap<OrderId, PendingOrder>>>,
    chained: RwLock<HashMap<OrderId, PendingOrder>>,
}

impl Orders {
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
            active: RwLock::new(HashMap::new()),
            chained: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_transactions(&self) -> Result<Vec<SecurityTransaction>> {
        let map = self.active.read().await;
        let transactions = map.iter().map(|kv| kv.1.to_owned()).collect();
        Ok(transactions)
    }

    pub async fn insert(&self, order_result: &OrderResult) -> Result<()> {
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
        let pending_map = self.pending.read().await;
        let mut results = vec![];
        let pending = pending_map
            .values()
            .flat_map(|v| v.values())
            .map(|p| p.to_owned());
        results.extend(pending);

        let chain_map = self.chained.read().await;
        let chained = chain_map.values().map(|p| p.to_owned());
        results.extend(chained);

        results
    }

    pub async fn get_pending_order(&self, security: &Security) -> Vec<PendingOrder> {
        let map_1 = self.pending.read().await;
        let mut results = vec![];
        if let Some(pds) = map_1.get(security) {
            let r = pds.values().map(|p| p.to_owned());
            results.extend(r)
        };

        let map_2 = self.chained.read().await;

        let s = security.to_owned();
        let filterd = map_2
            .values()
            .filter(|po| match po.order.to_owned() {
                NewOrder::Market(o) => o.security == s,
                NewOrder::Limit(o) => o.security == s,
                NewOrder::OCA(o) => o.limit_orders.iter().any(|l| l.security == s),
                NewOrder::StopLimitMarket(o) => o
                    .one_cancels_other
                    .limit_orders
                    .iter()
                    .any(|l| l.security == s),
            })
            .map(|p| p.to_owned());

        results.extend(filterd);

        results
    }

    pub async fn remove(&self, pending_order: &PendingOrder) -> Result<()> {
        let NewOrder::Limit(_) = pending_order.order.to_owned() else {
            let mut map = self.chained.write().await;
            let Some(_) = map.remove(&pending_order.order_id) else {
                bail!("order doesn't exist")
            };

            return Ok(());
        };

        let security = get_security(&pending_order.order);
        let mut map = self.pending.write().await;
        let Some(security_orders) = map.get_mut(security) else {
            bail!("order doesn't exist");
        };

        let oder_id = &pending_order.order_id;
        let Some(_) = security_orders.remove(oder_id) else {
            bail!("order doesn't exist");
        };

        if security_orders.is_empty() {
            map.remove(security);
        }

        Ok(())
    }

    async fn handle_filled(&self, filled_order: &FilledOrder) -> Result<()> {
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

    async fn handle_pending(&self, pending_order: &PendingOrder) -> Result<()> {
        let order_id = pending_order.order_id.to_owned();
        if let NewOrder::Market(_) = pending_order.order {
            bail!("market orders should immidiatly be executed")
        }

        let NewOrder::Limit(o) = pending_order.order.to_owned() else {
            let mut map = self.chained.write().await;
            map.insert(order_id, pending_order.to_owned());
            return Ok(());
        };

        let mut map = self.pending.write().await;
        if let Some(m) = map.get_mut(&o.security) {
            m.insert(pending_order.order_id.to_owned(), pending_order.to_owned());
            return Ok(());
        }
        let mut m = HashMap::new();
        m.insert(pending_order.order_id.to_owned(), pending_order.to_owned());
        map.insert(o.security.to_owned(), m);

        Ok(())
    }
}

fn get_security(order: &NewOrder) -> &Security {
    match order {
        NewOrder::Market(o) => &o.security,
        NewOrder::Limit(o) => &o.security,
        NewOrder::StopLimitMarket(o) => &o.market.security,
        NewOrder::OCA(o) => todo!(),
    }
}
