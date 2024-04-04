use crate::models::{
    price::{common::Resolution, price_history::PriceHistory, quote::Quote},
    security::Security,
};
use async_trait::async_trait;

#[async_trait]
pub trait QouteProvider {
    async fn get_quote(&self, security: &Security) -> Result<Quote, crate::error::Error>;
}

#[async_trait]
pub trait DataProvider {
    async fn get_data(
        &self,
        security: &Security,
        resolution: Resolution,
        lookback: u32,
    ) -> PriceHistory;
}
