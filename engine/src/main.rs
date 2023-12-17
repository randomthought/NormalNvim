use domain::{
    broker::Broker,
    engine::Engine,
    portfolio::Portfolio,
    risk::{config::RiskEngineConfig, risk_engine::RiskEngine},
    strategy::{Algorithm, StrategyEngine},
};
use engine::{
    algorithms::fake_algo::FakeAlgo,
    data_providers::polygon::{self, parser::PolygonParser, stream_client::create_stream},
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let api_key = env::var("API_KEY").unwrap();
    let quite_provider = polygon::api_client::ApiClient::new(api_key.clone(), client);
    let quite_provider_ = Arc::new(quite_provider);

    let broker = Broker::new(Decimal::from_u64(100_000).unwrap());
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

    let subscription = "A.*";
    let data_stream = create_stream(&api_key, &subscription).unwrap();
    let parser = Box::new(PolygonParser::new());
    let mut engine = Engine::new(strategy_engine, parser, data_stream);
    engine.runner().await.unwrap();
}
