use async_trait::async_trait;
use models::{
    price::{common::Resolution, price_history::PriceHistory, quote::Quote},
    security::Security,
};

#[async_trait]
pub trait QouteProvider {
    async fn get_quote(&self, security: &Security) -> Result<Quote, models::error::Error>;
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
