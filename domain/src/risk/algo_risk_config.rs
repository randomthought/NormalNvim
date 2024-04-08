use derive_builder::Builder;
use getset::Getters;
use models::strategy::common::StrategyId;
use rust_decimal::Decimal;
use traits::strategy::algorithm::Strategy;

#[derive(Builder, Getters, Clone, Copy)]
#[builder(setter(prefix = "with", strip_option))]
// #[getset(get = "pub")]
pub struct AlgorithmRiskConfig {
    pub strategy_id: StrategyId,
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
