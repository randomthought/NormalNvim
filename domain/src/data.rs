use crate::models::{
    price::{PriceHistory, Quote, Resolution},
    security::Security,
};
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[async_trait]
pub trait QouteProvider {
    async fn get_quote(&self, security: &Security) -> Result<Quote>;
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
