use crate::event::model::Market;
use crate::{event::model::Signal, models::order::OrderResult};
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[async_trait]
pub trait Algorithm {
    fn get_id(&self) -> String;
    async fn on_data(&self, market: &Market) -> Result<Option<Signal>>;
    async fn on_order(&self, order_result: &OrderResult) -> Result<()>;
}
