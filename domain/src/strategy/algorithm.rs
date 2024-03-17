use crate::event::model::Signal;
use async_trait::async_trait;

use super::algo_event::AlgoEvent;

pub type StrategyId = &'static str;

#[async_trait]
pub trait Algorithm {
    fn strategy_id(&self) -> StrategyId;
    async fn on_event<'a>(
        &self,
        algo_event: AlgoEvent<'a>,
    ) -> Result<Option<Signal>, crate::error::Error>;
}
