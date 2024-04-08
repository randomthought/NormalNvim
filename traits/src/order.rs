use async_trait::async_trait;
use models::orders::{
    common::OrderId, new_order::NewOrder, order_result::OrderResult, pending_order::PendingOrder,
    security_position::SecurityPosition,
};
use rust_decimal::Decimal;

#[async_trait]
pub trait Account {
    // TODO: think of making the return type Result<Box<Decimal>>
    async fn get_account_balance(&self) -> Result<Decimal, models::error::Error>;
    async fn get_buying_power(&self) -> Result<Decimal, models::error::Error>;
}

// Model based on https://developer.tdameritrade.com/account-access/apis
// TODO: model errors here https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/order-errors
#[async_trait]
pub trait OrderReader {
    async fn get_positions(&self) -> Result<Vec<SecurityPosition>, models::error::Error>;
    async fn get_pending_orders(&self) -> Result<Vec<PendingOrder>, models::error::Error>;
}

#[async_trait]
pub trait OrderManager: OrderReader {
    async fn place_order(&self, order: &NewOrder) -> Result<OrderResult, models::error::Error>;
    async fn update(
        &self,
        order_ticket: &PendingOrder,
    ) -> Result<OrderResult, models::error::Error>;
    // TODO: don't you think having the ID should be good enought to cancel the order?
    async fn cancel(&self, order: &OrderId) -> Result<OrderResult, models::error::Error>;
}
