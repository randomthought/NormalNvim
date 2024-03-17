use crate::models::order::SecurityPosition;
use async_trait::async_trait;
use rust_decimal::Decimal;

use super::algorithm::StrategyId;

#[async_trait]
pub trait StrategyPortfolio {
    async fn get_profit(&self, strategy_id: StrategyId) -> Result<Decimal, crate::error::Error>;
    async fn get_holdings(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<SecurityPosition>, crate::error::Error>;
}
