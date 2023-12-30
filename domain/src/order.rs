use super::models::order::{Order, OrderResult, PendingOrder};
use anyhow::Result;
use async_trait::async_trait;
use rust_decimal::Decimal;

#[async_trait]
pub trait Account {
    // TODO: think of making the return type Result<Box<Decimal>>
    async fn get_account_balance(&self) -> Result<Decimal>;
    async fn get_buying_power(&self) -> Result<Decimal>;
}

// Model based on https://developer.tdameritrade.com/account-access/apis
// TODO: model errors here https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/order-errors
#[async_trait]
pub trait OrderReader {
    async fn open_orders(&self) -> Result<Vec<OrderResult>>;
    async fn pending_orders(&self) -> Result<Vec<OrderResult>>;
}

#[async_trait]
pub trait OrderManager: OrderReader {
    async fn place_order(&self, order: &Order) -> Result<OrderResult>;
    // TODO: model order not exisiting error
    async fn update(&self, order_ticket: &PendingOrder) -> Result<()>;
    async fn cancel(&self, order: &PendingOrder) -> Result<()>;
}
