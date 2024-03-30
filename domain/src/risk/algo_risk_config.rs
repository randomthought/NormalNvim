use derive_builder::Builder;
use rust_decimal::Decimal;

use crate::strategy::algorithm::{Strategy, StrategyId};

#[derive(Builder, Clone, Copy)]
#[builder(setter(prefix = "with", strip_option))]
pub struct AlgorithmRiskConfig {
    strategy_id: StrategyId,
    #[builder(default)]
    pub starting_balance: Decimal,
    #[builder(default)]
    pub max_open_trades: Option<u32>,
    #[builder(default)]
    pub max_portfolio_loss: Option<f64>,
    #[builder(default = "Some(1f64)")]
    pub max_portfolio_risk: Option<f64>,
    #[builder(default)]
    pub max_risk_per_trade: Option<f64>,
    #[builder(default)]
    pub max_pending_orders: Option<u32>,
}

impl AlgorithmRiskConfig {
    pub fn builder() -> AlgorithmRiskConfigBuilder {
        AlgorithmRiskConfigBuilder::default()
    }
}

impl Strategy for AlgorithmRiskConfig {
    fn strategy_id(&self) -> StrategyId {
        self.strategy_id
    }
}
