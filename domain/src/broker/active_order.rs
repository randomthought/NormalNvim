use std::i64;

use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::models::{
    order::{OrderDetails, Quantity, Side},
    price::Price,
    security::Security,
};

enum OrderState {
    Filled,
    Closed,
}

#[derive(Debug, Clone)]
pub struct ActiveOrder {
    pub security: Security,

    // TODO: maybe keep current order state here?
    // TODO: sort by datetime
    pub order_details: Vec<OrderDetails>,
}

pub struct SecurityPosition {
    pub side: Side,
    pub quantity: Quantity,
}

impl ActiveOrder {
    pub fn new(security: Security) -> Self {
        Self {
            security,
            order_details: Vec::new(),
        }
    }
    pub fn get_position(&self) -> Option<SecurityPosition> {
        let quantity: i64 = self.order_details.iter().fold(0, |acc, ad| match ad.side {
            Side::Long => acc + i64::from_u64(ad.quantity).unwrap(),
            Side::Short => acc - i64::from_u64(ad.quantity).unwrap(),
        });

        if quantity > 0 {
            return Some(SecurityPosition {
                quantity: u64::from_i64(quantity).unwrap(),
                side: Side::Long,
            });
        } else if quantity < 0 {
            return Some(SecurityPosition {
                quantity: u64::from_i64(quantity * -1).unwrap(),
                side: Side::Short,
            });
        }

        None
    }

    pub fn insert(&mut self, order_details: OrderDetails) {
        self.order_details.push(order_details);
    }
}
