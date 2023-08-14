use std::sync::Arc;

use domain::{
    engine::Engine,
    event::{channel_pipe::ChannelPipe, event::Pipe},
    risk::{RiskEngine, RiskEngineConfig},
    strategy::{Algorithm, StrategyEngine},
};
use engine::FakeOrderManager;

#[tokio::main]
async fn main() {
    let pipe = ChannelPipe::default();
    let risk_engine_config = RiskEngineConfig {
        max_portfolio_risk: 0.10,
        max_risk_per_trade: 0.05,
        max_open_trades: None,
    };

    let bpip: Arc<Box<dyn Pipe + Send + Sync>> = Arc::new(Box::new(pipe));

    let order_manager = FakeOrderManager {};

    let algorithms: Vec<&(dyn Algorithm + Send + Sync)> = vec![];

    tokio::spawn(async move {
        let risk_engine = RiskEngine::new(risk_engine_config, Box::new(order_manager));
        let strategy_engine = StrategyEngine::new(algorithms, bpip.clone());
        let mut algo_engine = Engine::new(strategy_engine, risk_engine, bpip.clone());
        algo_engine.runner().await.unwrap();
    });
}
