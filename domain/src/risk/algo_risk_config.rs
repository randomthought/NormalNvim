use derive_builder::Builder;
use rust_decimal::Decimal;

#[derive(Builder)]
pub struct AlgorithmRiskConfig {
    #[builder(setter(prefix = "with"), default)]
    starting_balance: Decimal,
    #[builder(setter(prefix = "with"))]
    max_open_trades: Option<u32>,
    #[builder(setter(prefix = "with"))]
    max_portfolio_loss: Option<f64>,
    #[builder(setter(prefix = "with"), default = "Some(1f64)")]
    max_portfolio_risk: Option<f64>,
    #[builder(setter(prefix = "with"))]
    max_risk_per_trade: Option<f64>,
    #[builder(setter(prefix = "with"))]
    max_pending_orders: Option<u32>,
}

impl AlgorithmRiskConfig {
    pub fn builder() -> AlgorithmRiskConfigBuilder {
        AlgorithmRiskConfigBuilder::default()
    }
}
