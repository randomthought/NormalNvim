use std::{pin::Pin, sync::Arc};

use domain::{
    engine::Engine,
    portfolio::Portfolio,
    risk::{config::RiskEngineConfig, risk_engine::RiskEngine},
    strategy::{Algorithm, StrategyEngine},
};
use engine::{
    algorithms::fake_algo::FakeAlgo,
    brokers::fake_broker::FakeBroker,
    data_providers::polygon::{self, stream_client::PolygonClient},
};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let api_key = "XXX..".to_owned();
    let polygon_client = PolygonClient::new(api_key.clone()).await.unwrap();
    let quite_provider = polygon::api_client::ApiClient::new(api_key.clone(), client);
    let quite_provider_ = Arc::new(quite_provider);

    let broker = FakeBroker {
        account_balance: 100000.0,
    };
    let broker_ = Arc::new(broker);
    let risk_engine_config = RiskEngineConfig {
        max_trade_portfolio_accumulaton: 0.10,
        max_portfolio_risk: 0.10,
        max_risk_per_trade: 0.005,
        max_open_trades: None,
    };

    let portfolio = Portfolio::new(broker_.clone(), broker_.clone(), quite_provider_.clone());
    let risk_egnine = RiskEngine::new(
        risk_engine_config,
        quite_provider_.clone(),
        broker_.clone(),
        Box::new(portfolio),
    );
    let risk_engine_ = Box::new(risk_egnine);

    let algorithms: Vec<Box<dyn Algorithm + Send + Sync>> = vec![Box::new(FakeAlgo {})];
    let strategy_engine = StrategyEngine::new(risk_engine_, algorithms);

    let mut engine = Engine::new(strategy_engine, Box::pin(polygon_client));
    engine.runner().await.unwrap();
}
