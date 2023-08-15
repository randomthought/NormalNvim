use futures_util::future;
use std::{io, sync::Arc};

use crate::{
    event::event::{EventHandler, Pipe},
    risk::RiskEngine,
    strategy::StrategyEngine,
};

pub struct Engine {
    strategy_engine: StrategyEngine,
    risk_engine: RiskEngine,
    pipe: Arc<Box<dyn Pipe + Send + Sync>>,
}

impl Engine {
    pub fn new(
        strategy_engine: StrategyEngine,
        risk_engine: RiskEngine,

        pipe: Arc<Box<dyn Pipe + Send + Sync>>,
    ) -> Self {
        Self {
            strategy_engine,
            risk_engine,
            pipe,
        }
    }

    pub async fn runner(&mut self) -> Result<(), io::Error> {
        while let Some(event) = self.pipe.recieve().await? {
            let f1 = self.strategy_engine.handle(event.clone());
            let f2 = self.risk_engine.handle(event);

            future::try_join_all(vec![f1, f2]).await?;
        }

        Ok(())
    }
}
