use futures_util::{Stream, StreamExt};
use std::{io, pin::Pin, sync::Arc};

use anyhow::{Context, Result};

use crate::{models::price::PriceHistory, strategy::StrategyEngine};

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
