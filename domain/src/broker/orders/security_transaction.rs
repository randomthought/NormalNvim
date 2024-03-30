use derive_builder::Builder;
use derive_getters::Getters;

use crate::models::{
    orders::{
        common::{OrderDetails, OrderId, Side},
        filled_order::FilledOrder,
        security_position::{HoldingDetail, SecurityPosition},
    },
    price::common::Price,
    security::Security,
};
use std::time::Duration;

// TODO: make struct private
#[derive(Debug, Builder, Getters, Clone)]
#[builder(setter(prefix = "with"))]
pub struct Transaction {
    pub order_id: OrderId,
    pub price: Price,
    pub order_details: OrderDetails,
    pub date_time: Duration,
}

impl Transaction {
    pub fn builder() -> TransactionBuilder {
        TransactionBuilder::default()
    }
}

#[derive(Debug, Clone)]
pub struct SecurityTransaction {
    pub security: Security,
    pub order_history: Vec<Transaction>,
}

impl SecurityTransaction {
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

    pub fn insert(&mut self, filled_order: &FilledOrder) -> Result<(), String> {
        if filled_order.security != self.security {
            return Err("security must match".into());
        }

        let transation = Transaction {
            order_id: filled_order.order_id.to_owned(),
            order_details: filled_order.order_details.to_owned(),
            date_time: filled_order.date_time,
            price: filled_order.price,
        };

        self.order_history.push(transation);

        Ok(())
    }
}

fn add_to_position(security_position: &mut SecurityPosition, transaction: &Transaction) {
    let hd = HoldingDetail::builder()
        .with_strategy_id(transaction.order_details().strategy_id)
        .with_quantity(transaction.order_details().quantity)
        .with_price(transaction.price().clone())
        .build()
        .unwrap();

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
        security_position.holding_details.clear();
        return;
    }

    if current_quantity > transaction.order_details.quantity {
        let hd = HoldingDetail::builder()
            .with_strategy_id(transaction.order_details().strategy_id)
            .with_quantity(current_quantity - transaction.order_details().quantity)
            .with_price(hd.price.to_owned())
            .build()
            .unwrap();

        security_position.holding_details.push(hd);
        return;
    }

    let ts = Transaction {
        order_id: transaction.order_id.to_owned(),
        date_time: transaction.date_time,
        price: transaction.price,
        order_details: OrderDetails {
            strategy_id: transaction.order_details.strategy_id,
            quantity: transaction.order_details.quantity - holding_detail.quantity,
            side: transaction.order_details.side,
        },
    };

    add_to_position(security_position, &ts)
}
