use domain::{
    engine::Engine,
    order::OrderManager,
    risk::{RiskEngine, RiskEngineConfig},
    strategy::{Algorithm, StrategyEngine},
};
use engine::{
    algorithms::fake_algo::FakeAlgo,
    brokers::fake_broker::FakeOrderManager,
    data_providers::polygon::{models::Aggregates, polygon::PolygonClient},
};
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let api_key = "XXXX...".to_owned();
    let polygon_client = PolygonClient::new(api_key).await.unwrap();

    let order_manager = FakeOrderManager {};
    let risk_engine_config = RiskEngineConfig {
        max_portfolio_risk: 0.10,
        max_risk_per_trade: 0.005,
        max_open_trades: None,
    };
    let risk_egnine = RiskEngine::new(risk_engine_config, Box::new(order_manager));

    let algorithms: Vec<Box<dyn Algorithm + Send + Sync>> = vec![Box::new(FakeAlgo {})];
    let strategy_engine = StrategyEngine::new(Box::new(risk_egnine), algorithms);
    let mut engine = Engine::new(strategy_engine, Box::pin(polygon_client));
    engine.runner().await.unwrap();
}
