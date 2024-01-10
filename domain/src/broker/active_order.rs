use std::{i64, time::Duration};

use anyhow::{bail, Result};
use rust_decimal::prelude::FromPrimitive;

use crate::models::{
    order::{FilledOrder, OrderDetails, OrderId, SecurityPosition, Side},
    security::Security,
};

// TODO: make struct private
#[derive(Debug, Clone)]
pub struct Transation {
    order_id: OrderId,
    order_details: OrderDetails,
    date_time: Duration,
}

#[derive(Debug, Clone)]
pub struct ActiveOrder {
    pub security: Security,
    pub order_history: Vec<Transation>,
}

impl ActiveOrder {
    pub fn new(security: Security) -> Self {
        Self {
            security,
            order_history: Vec::new(),
        }
    }
    pub fn get_position(&self) -> Option<SecurityPosition> {
        let quantity: i64 =
            self.order_history
                .iter()
                .fold(0, |acc, ad| match ad.order_details.side {
                    Side::Long => acc + i64::from_u64(ad.order_details.quantity).unwrap(),
                    Side::Short => acc - i64::from_u64(ad.order_details.quantity).unwrap(),
                });

        if quantity == 0 {
            return None;
        }

        todo!()

        // Some(SecurityPosition {
        //     security: self.security.to_owned(),
        //     quantity: u64::from_i64(quantity.abs()).unwrap(),
        //     side: if quantity < 0 {
        //         Side::Short
        //     } else {
        //         Side::Long
        //     },
        // })
    }

    pub fn insert(&mut self, filled_order: &FilledOrder) -> Result<()> {
        if filled_order.security != self.security {
            bail!("security must match");
        }

        let transation = Transation {
            order_id: filled_order.order_id.to_owned(),
            order_details: filled_order.order_details.to_owned(),
            date_time: filled_order.date_time,
        };

        self.order_history.push(transation);

        Ok(())
    }
}
