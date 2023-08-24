use futures_util::{future, Stream, StreamExt};
use std::{io, pin::Pin, sync::Arc};

use crate::risk::RiskEngine;
use crate::{models::price::PriceHistory, strategy::StrategyEngine};

pub struct Engine {
    strategy_engine: StrategyEngine,
    market_stream: Pin<Box<dyn Stream<Item = Result<PriceHistory, io::Error>>>>,
}

impl Engine {
    pub fn new(
        strategy_engine: StrategyEngine,
        market_stream: Pin<Box<dyn Stream<Item = Result<PriceHistory, io::Error>>>>,
    ) -> Self {
        Self {
            strategy_engine,
            market_stream,
        }
    }

    pub async fn runner(&mut self) -> Result<(), io::Error> {
        while let some = self.market_stream.next().await {
            match some {
                Some(Ok(item)) => print!("{}", item.security.ticker),
                Some(Err(err)) => return Err(err),
                _ => (),
            }
        }

        Ok(())
    }
}
