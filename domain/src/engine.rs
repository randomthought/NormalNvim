use futures_util::{future, Stream, StreamExt};
use std::{io, pin::Pin, sync::Arc};

use crate::risk::RiskEngine;
use crate::{models::price::PriceHistory, strategy::StrategyEngine};

pub struct MarketStream {}

impl Stream for MarketStream {
    type Item = PriceHistory;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

pub struct Engine {
    strategy_engine: StrategyEngine,
    market_stream: Pin<Box<MarketStream>>,
}

impl Engine {
    pub fn new(strategy_engine: StrategyEngine, market_stream: Pin<Box<MarketStream>>) -> Self {
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
