use std::collections::HashMap;

use crate::models::{
    orders::{common::OrderId, pending_order::PendingOrder},
    security::Security,
};

pub enum PendingKey {
    SecurityKey(Security),
    OrderIdKey(OrderId),
    None,
}

#[derive(Default)]
pub struct Pending {
    pending_by_security: HashMap<Security, HashMap<OrderId, PendingOrder>>,
    pending_by_order_id: HashMap<OrderId, PendingOrder>,
}

impl Pending {
    pub fn get(&self, pending_key: PendingKey) -> Vec<PendingOrder> {
        match pending_key {
            PendingKey::OrderIdKey(x) => self
                .pending_by_order_id
                .get(&x)
                .iter()
                .map(|&p| p.to_owned())
                .collect(),
            PendingKey::SecurityKey(x) => self
                .pending_by_security
                .get(&x)
                .iter()
                .flat_map(|m| m.values())
                .map(|p| p.to_owned())
                .collect(),
            PendingKey::None => self
                .pending_by_order_id
                .values()
                .map(|p| p.to_owned())
                .collect(),
        }
    }

    pub fn remove(&mut self, pending_key: PendingKey) {
        match pending_key {
            PendingKey::OrderIdKey(x) => {
                let Some(p) = self.pending_by_order_id.remove(&x) else {
                    return;
                };

                if let Some(smap) = self.pending_by_security.get_mut(&p.order.get_security()) {
                    smap.remove(&x);
                }
            }
            PendingKey::SecurityKey(x) => {
                let Some(map) = self.pending_by_security.get_mut(&x) else {
                    return;
                };

                for k in map.keys() {
                    self.pending_by_order_id.remove(k);
                }

                self.pending_by_security.remove(&x);
            }
            PendingKey::None => (),
        };
    }

    pub fn update(&mut self, pending_order: PendingOrder) {
        let security = pending_order.order.get_security().to_owned();
        let order_id = pending_order.order_id.to_owned();
        match self.pending_by_security.get_mut(&security) {
            Some(id_map) => id_map.insert(order_id.to_owned(), pending_order.to_owned()),
            None => {
                let mut map = HashMap::new();
                map.insert(order_id.to_owned(), pending_order.to_owned())
            }
        };

        self.pending_by_order_id
            .insert(order_id.to_owned(), pending_order.to_owned());
    }
}
