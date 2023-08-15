use domain::{
    engine::Engine,
    risk::{RiskEngine, RiskEngineConfig},
    strategy::{Algorithm, StrategyEngine},
};
use engine::{
    algorithms::fake_algo::FakeAlgo, brokers::fake_broker::FakeOrderManager,
    data_providers::fake_provider::FakePriceHistoryStream,
};

#[tokio::main]
async fn main() {
    let risk_engine_config = RiskEngineConfig {
        max_portfolio_risk: 0.10,
        max_risk_per_trade: 0.05,
        max_open_trades: None,
    };

    let order_manager = FakeOrderManager {};

    let algorithms: Vec<Box<dyn Algorithm + Send + Sync>> = vec![Box::new(FakeAlgo {})];

    let risk_engine = RiskEngine::new(risk_engine_config, Box::new(order_manager));
    let strategy_engine = StrategyEngine::new(Box::new(risk_engine), algorithms);
    let market_stream = Box::pin(FakePriceHistoryStream { max: 10 });
    let mut algo_engine = Engine::new(strategy_engine, market_stream);
    algo_engine.runner().await.unwrap();
}
