use std::{i64, time::Duration};

use anyhow::{bail, Result};
use rust_decimal::prelude::FromPrimitive;

use crate::models::{
    order::{FilledOrder, HoldingDetail, OrderDetails, OrderId, SecurityPosition, Side},
    price::Price,
    security::Security,
};

// TODO: make struct private
#[derive(Debug, Clone)]
pub struct Transation {
    order_id: OrderId,
    price: Price,
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

    // TODO: write unit test
    pub fn get_position(&self) -> Option<SecurityPosition> {
        let mut security_position = SecurityPosition {
            security: self.security.to_owned(),
            side: Side::Long,
            holding_details: vec![],
        };

        self.order_history
            .iter()
            .for_each(|transaction| add_to_position(&mut security_position, transaction));

        if security_position.get_quantity() == 0 {
            return None;
        }

        Some(security_position)
    }

    pub fn insert(&mut self, filled_order: &FilledOrder) -> Result<()> {
        if filled_order.security != self.security {
            bail!("security must match");
        }

        let transation = Transation {
            order_id: filled_order.order_id.to_owned(),
            order_details: filled_order.order_details.to_owned(),
            date_time: filled_order.date_time,
            price: filled_order.price,
        };

        self.order_history.push(transation);

        Ok(())
    }
}

fn add_to_position(security_position: &mut SecurityPosition, transaction: &Transation) {
    let hd = to_holding_details(transaction);
    let current_quantity = security_position.get_quantity();
    let Some(holding_detail) = security_position.holding_details.pop() else {
        security_position.side = transaction.order_details.side;
        security_position.holding_details.push(hd);
        return;
    };

    if security_position.side == transaction.order_details.side {
        security_position.holding_details.push(holding_detail);
        security_position.holding_details.push(hd.to_owned());
        return;
    }

    if current_quantity == hd.quantity {
        return;
    }

    if current_quantity > transaction.order_details.quantity {
        let hd = HoldingDetail {
            quantity: current_quantity - transaction.order_details.quantity,
            price: hd.price.to_owned(),
        };
        security_position.holding_details.push(hd);
        return;
    }

    let ts = Transation {
        order_id: transaction.order_id.to_owned(),
        date_time: transaction.date_time,
        price: transaction.price,
        order_details: OrderDetails {
            quantity: transaction.order_details.quantity - holding_detail.quantity,
            side: transaction.order_details.side,
        },
    };

    add_to_position(security_position, &ts)
}

fn to_holding_details(transation: &Transation) -> HoldingDetail {
    HoldingDetail {
        quantity: transation.order_details.quantity,
        price: transation.price,
    }
}
