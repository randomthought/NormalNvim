use super::models::order::{Order, OrderResult, OrderTicket};
use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait Account {
    async fn get_account_balance(&self) -> Result<f64, io::Error>;
    async fn get_buying_power(&self) -> Result<f64, io::Error>;
}

// Model based on https://developer.tdameritrade.com/account-access/apis
// TODO: model errors here https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/order-errors
#[async_trait]
pub trait OrderReader {
    // TODO: Please look more into your use of lifetimes here.
    async fn get(&self) -> Result<Order, io::Error>;
    async fn orders(&self) -> Result<Vec<&OrderResult>, io::Error>;
}

#[async_trait]
pub trait OrderManager: OrderReader {
    async fn place_order(&self, order: &Order) -> Result<OrderResult, io::Error>;
    async fn update(&self, order_ticket: &OrderTicket) -> Result<(), io::Error>;
    async fn cancel(&self, order: &OrderTicket) -> Result<(), io::Error>;
}
