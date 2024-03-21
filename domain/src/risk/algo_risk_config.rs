use derive_builder::Builder;
use rust_decimal::Decimal;

use crate::strategy::algorithm::StrategyId;

#[derive(Builder, Clone, Copy)]
pub struct AlgorithmRiskConfig {
    #[builder(setter(prefix = "with"), default)]
    pub strategy_id: StrategyId,
    #[builder(setter(prefix = "with"), default)]
    pub starting_balance: Decimal,
    #[builder(setter(prefix = "with", strip_option))]
    pub max_open_trades: Option<u32>,
    #[builder(setter(prefix = "with", strip_option))]
    pub max_portfolio_loss: Option<f64>,
    #[builder(setter(prefix = "with", strip_option), default = "Some(1f64)")]
    pub max_portfolio_risk: Option<f64>,
    #[builder(setter(prefix = "with", strip_option))]
    pub max_risk_per_trade: Option<f64>,
    #[builder(setter(prefix = "with", strip_option))]
    pub max_pending_orders: Option<u32>,
}

impl AlgorithmRiskConfig {
    pub fn builder() -> AlgorithmRiskConfigBuilder {
        AlgorithmRiskConfigBuilder::default()
    }
}
