use crate::models::{
    price::{PriceHistory, Quote, Resolution},
    security::Security,
};
use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait QouteProvider {
    async fn get_quote(&self, security: &Security) -> Result<Quote, io::Error>;
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
