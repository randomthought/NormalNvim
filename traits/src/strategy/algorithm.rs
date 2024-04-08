use async_trait::async_trait;
use models::strategy::{algo_event::AlgoEvent, common::StrategyId, signal::Signal};

pub trait Strategy {
    fn strategy_id(&self) -> StrategyId;
}

#[async_trait]
pub trait Algorithm {
    async fn on_event(&self, algo_event: AlgoEvent)
        -> Result<Option<Signal>, models::error::Error>;
}
