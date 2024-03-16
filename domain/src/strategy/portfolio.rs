use crate::models::{order::SecurityPosition, price::Price, security::Security};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use rust_decimal::Decimal;

use super::algorithm::StrategyId;

#[async_trait]
pub trait StrategyPortfolio {
    async fn get_profit(&self, strategy_id: StrategyId) -> Result<Decimal>;
    async fn get_holdings(&self, strategy_id: StrategyId) -> Result<Vec<SecurityPosition>>;
}
