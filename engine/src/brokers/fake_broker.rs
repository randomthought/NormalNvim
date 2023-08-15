use std::io;

use async_trait::async_trait;
use domain::{
    models::order::{Order, OrderResult, OrderTicket},
    order::{OrderManager, OrderReader},
};

pub struct FakeOrderManager {}

#[async_trait]
impl OrderReader for FakeOrderManager {
    async fn get(&self) -> Result<Order, io::Error> {
        todo!()
    }
    async fn orders(&self) -> Result<Vec<&OrderResult>, io::Error> {
        todo!()
    }
}

#[async_trait]
impl OrderManager for FakeOrderManager {
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
