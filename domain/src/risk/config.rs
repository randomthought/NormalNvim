use color_eyre::eyre::{ensure, Result};

enum TradingState {
    Active, // trading is enabled
    // TODO: add an update order type this might be good for only accepting modified orders and not trading
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

#[derive(Debug, Clone, Copy)]
pub struct RiskEngineConfig {
    pub max_trade_portfolio_accumulaton: f64,
    // TODO: add functionality to check portfolio risk on orders
    pub max_portfolio_risk: f64,
    pub max_open_trades: Option<u32>,
    // TODO: nautilus_trader has max_order_submit_rate and max_order_modify_rate. Maybe it's worth having
}

impl RiskEngineConfig {
    pub fn new(max_portfolio_risk: f64, max_trade_portfolio_accumulaton: f64) -> Result<Self> {
        ensure!(
            (0.0..=1.0).contains(&max_portfolio_risk),
            "{dbg!(max_portfolio_risk)} has to be between a value 0 and 1".to_owned()
        );

        ensure!(
            (0.0..=1.0).contains(&max_trade_portfolio_accumulaton),
            "{dbg!(max_trade_portfolio_accumulaton)} has to be between a value 0 and 1".to_owned()
        );

        Ok(Self {
            max_portfolio_risk,
            max_open_trades: None,
            max_trade_portfolio_accumulaton,
        })
    }
}
