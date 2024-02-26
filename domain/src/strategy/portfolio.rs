use crate::models::{order::SecurityPosition, price::Price, security::Security};
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[async_trait]
pub trait AlgorithmPortfolio {
    async fn get_balance(&self) -> Result<Price>;
    async fn get_holdings(&self) -> Result<Vec<SecurityPosition>>;
    async fn get_holding(&self, security: &Security) -> Result<Option<SecurityPosition>>;
}
