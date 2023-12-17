use crate::{
    models::order::{Order, OrderResult, OrderTicket},
    order::{Account, OrderManager, OrderReader},
};
use anyhow::Result;
use async_trait::async_trait;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::sync::RwLock;

pub struct Broker {
    account_balance: RwLock<Decimal>,
    orders: Vec<OrderResult>,
    commissions_per_share: Decimal,
}

impl Broker {
    pub fn new(account_balance: Decimal) -> Self {
        let commissions_per_share = Decimal::from_f64(0.0).unwrap();
        Self {
            account_balance: RwLock::new(account_balance),
            orders: vec![],
            commissions_per_share,
        }
    }
}

#[async_trait]
impl Account for Broker {
    async fn get_account_balance(&self) -> Result<Decimal> {
        let results = self.account_balance.read().unwrap();
        Ok(*results)
    }
    async fn get_buying_power(&self) -> Result<Decimal> {
        let results = self.account_balance.read().unwrap();
        Ok(*results)
    }
}

#[async_trait]
impl OrderReader for Broker {
    async fn orders(&self) -> Result<Vec<OrderResult>> {
        Ok(self.orders.clone())
    }
}

#[async_trait]
impl OrderManager for Broker {
    async fn place_order(&self, order: &Order) -> Result<OrderResult> {
        match order {
            Order::Market(o) => {}
            Order::Limit(o) => {}
            Order::StopLimitMarket(o) => {}
        }
        todo!()
    }
    async fn update(&self, order_ticket: &OrderTicket) -> Result<()> {
        todo!()
    }

    async fn cancel(&self, order: &OrderTicket) -> Result<()> {
        todo!()
    }
}
