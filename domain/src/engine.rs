use crate::{models::price::PriceHistory, strategy::StrategyEngine};
use anyhow::Result;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

pub struct Engine {
    strategy_engine: StrategyEngine,
    market_stream: Pin<Box<dyn Stream<Item = Result<PriceHistory>>>>,
}

impl Engine {
    pub fn new(
        strategy_engine: StrategyEngine,
        market_stream: Pin<Box<dyn Stream<Item = Result<PriceHistory>>>>,
    ) -> Self {
        Self {
            strategy_engine,
            market_stream,
        }
    }

    pub async fn runner(&mut self) -> Result<()> {
        loop {
            match self.market_stream.next().await {
                Some(Ok(price_history)) => self.strategy_engine.process(price_history).await?,
                Some(Err(err)) => return Err(err),
                _ => (),
            }
        }
    }
}
