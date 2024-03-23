use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::models::orders::{
    common::OrderId, new_order::NewOrder, order_result::OrderResult, pending_order::PendingOrder,
    security_position::SecurityPosition,
};

#[async_trait]
pub trait Account {
    // TODO: think of making the return type Result<Box<Decimal>>
    async fn get_account_balance(&self) -> Result<Decimal, crate::error::Error>;
    async fn get_buying_power(&self) -> Result<Decimal, crate::error::Error>;
}

// Model based on https://developer.tdameritrade.com/account-access/apis
// TODO: model errors here https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/order-errors
#[async_trait]
pub trait OrderReader {
    async fn get_positions(&self) -> Result<Vec<SecurityPosition>, crate::error::Error>;
    async fn get_pending_orders(&self) -> Result<Vec<OrderResult>, crate::error::Error>;
}

#[async_trait]
pub trait OrderManager: OrderReader {
    async fn place_order(&self, order: &NewOrder) -> Result<OrderResult, crate::error::Error>;
    async fn update(&self, order_ticket: &PendingOrder)
        -> Result<OrderResult, crate::error::Error>;
    // TODO: don't you think having the ID should be good enought to cancel the order?
    async fn cancel(&self, order: &OrderId) -> Result<OrderResult, crate::error::Error>;
}
