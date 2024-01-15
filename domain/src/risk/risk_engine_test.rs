use std::sync::Arc;

use crate::{broker::broker::Broker, risk::config};

struct Setup;

impl Setup {
    pub fn new() -> Self {
        todo!()
    }

    pub fn create_broker(risk_engine: config::RiskEngineConfig) -> Broker {
        todo!()
    }
}

#[cfg(test)]
#[tokio::test]
async fn reject_trade_on_halt() {
    let setup = Setup::new();

    let risk_engine_config = config::RiskEngineConfig::new(1.0, 1.0, 1.0);
    todo!()
}

#[tokio::test]
async fn reject_trade_on_portfolio_risk() {
    todo!()
}

#[tokio::test]
async fn reject_trade_on_max_open_trades() {
    todo!()
}
