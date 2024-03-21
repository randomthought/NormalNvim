use derive_builder::Builder;
use rust_decimal::Decimal;

use crate::strategy::algorithm::{Strategy, StrategyId};

#[derive(Builder, Clone, Copy)]
pub struct AlgorithmRiskConfig {
    #[builder(private)]
    strategy_id: StrategyId,
    #[builder(default, setter(prefix = "with"))]
    pub starting_balance: Decimal,
    #[builder(default, setter(prefix = "with", strip_option))]
    pub max_open_trades: Option<u32>,
    #[builder(default, setter(prefix = "with", strip_option))]
    pub max_portfolio_loss: Option<f64>,
    #[builder(setter(prefix = "with", strip_option), default = "Some(1f64)")]
    pub max_portfolio_risk: Option<f64>,
    #[builder(default, setter(prefix = "with", strip_option))]
    pub max_risk_per_trade: Option<f64>,
    #[builder(default, setter(prefix = "with", strip_option))]
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

impl AlgorithmRiskConfigBuilder {
    pub fn with_strategy_id(&mut self, strategy_id: StrategyId) -> &mut Self {
        self.strategy_id = Some(strategy_id);
        self
    }
}
