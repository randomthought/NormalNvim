use std::io;

use async_trait::async_trait;
use domain::{
    models::order::{Order, OrderResult, OrderTicket},
    order::{Account, OrderManager, OrderReader},
};

pub struct FakeBroker {
    pub account_balance: f64,
}

#[async_trait]
impl Account for FakeBroker {
    async fn get_account_balance(&self) -> Result<f64, io::Error> {
        Ok(self.account_balance)
    }
    async fn get_buying_power(&self) -> Result<f64, io::Error> {
        Ok(self.account_balance)
    }
}

#[async_trait]
impl OrderReader for FakeBroker {
    async fn get(&self) -> Result<Order, io::Error> {
        todo!()
    }
    async fn orders(&self) -> Result<Vec<&OrderResult>, io::Error> {
        todo!()
    }
}

#[async_trait]
impl OrderManager for FakeBroker {
    async fn place_order(&self, order: &Order) -> Result<OrderResult, io::Error> {
        todo!()
    }
    async fn update(&self, order_ticket: &OrderTicket) -> Result<(), io::Error> {
        todo!()
    }

    async fn cancel(&self, order: &OrderTicket) -> Result<(), io::Error> {
        todo!()
    }
}
