use std::i64;

use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::models::{
    order::{OrderDetails, Quantity, Side},
    price::{Price, Quote},
    security::Security,
};

enum OrderState {
    Filled,
    Closed,
}

#[derive(Debug, Clone)]
pub struct ActiveOrder {
    pub security: Security,

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

    pub fn cost(&self, quote: &Quote) -> Price {
        let mut sorted = self.order_details.to_vec();
        sorted.sort_by_key(|o| o.datetime);

        let init = Decimal::from_u64(0).unwrap();
        self.order_details.iter().fold(init, |acc, ad| {
            let q = Decimal::from_u64(ad.quantity).unwrap();
            match ad.side {
                Side::Long => acc + (q * (ad.price - quote.ask)),
                Side::Short => acc + (q * (quote.bid - ad.price)),
            }
        })
    }
}
