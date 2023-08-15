use futures_util::{future, Stream, StreamExt};
use std::{io, pin::Pin, sync::Arc};

use crate::risk::RiskEngine;
use crate::{models::price::PriceHistory, strategy::StrategyEngine};

pub struct Engine {
    strategy_engine: StrategyEngine,
    market_stream: Pin<Box<dyn Stream<Item = PriceHistory>>>,
}

impl Engine {
    pub fn new(
        strategy_engine: StrategyEngine,
        market_stream: Pin<Box<dyn Stream<Item = PriceHistory>>>,
    ) -> Self {
        Self {
            strategy_engine,
            market_stream,
        }
    }

    pub async fn runner(&mut self) -> Result<(), io::Error> {
        while let Some(item) = self.market_stream.next().await {
            self.strategy_engine.process(item).await?;
        }
        Ok(())
    }
}
