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
        loop {
            match self.market_stream.next().await {
                Some(Ok(item)) => println!("{:?}", item),
                Some(Err(err)) => return Err(err),
                _ => (),
            }
        }
    }
}
