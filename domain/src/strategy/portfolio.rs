use crate::models::{order::SecurityPosition, price::Price, security::Security};
use async_trait::async_trait;
use color_eyre::eyre::Result;

use super::algorithm::StrategyId;

#[async_trait]
pub trait StrategyPortfolio {
    async fn get_balance(&self, strategy_id: &StrategyId) -> Result<Price>;
    async fn get_holdings(&self, strategy_id: &StrategyId) -> Result<Vec<SecurityPosition>>;
}
