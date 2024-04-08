use async_trait::async_trait;
use models::{
    orders::{pending_order::PendingOrder, security_position::SecurityPosition},
    strategy::common::StrategyId,
};
use rust_decimal::Decimal;

#[async_trait]
pub trait StrategyPortfolio {
    async fn get_profit(&self, strategy_id: StrategyId) -> Result<Decimal, models::error::Error>;
    async fn get_security_positions(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<SecurityPosition>, models::error::Error>;
    async fn get_pending(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<PendingOrder>, models::error::Error>;
}
