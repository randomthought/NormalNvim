use crate::{models::price::PriceHistory, strategy::StrategyEngine};
use anyhow::Result;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

pub trait Parser {
    fn parse(&self, data: &str) -> Result<Box<dyn Iterator<Item = PriceHistory>>>;
}

pub struct Engine {
    strategy_engine: StrategyEngine,
    parser: Box<dyn Parser>,
    data_stream: Pin<Box<dyn Stream<Item = Result<String>>>>,
}

impl Engine {
    pub fn new(
        strategy_engine: StrategyEngine,
        parser: Box<dyn Parser>,
        data_stream: Pin<Box<dyn Stream<Item = Result<String>>>>,
    ) -> Self {
        Self {
            strategy_engine,
            parser,
            data_stream,
        }
    }

    pub async fn runner(&mut self) -> Result<()> {
        loop {
            match self.data_stream.next().await {
                Some(Ok(data)) => self.process_data(&data).await?,
                Some(Err(err)) => return Err(err),
                _ => (),
            }
        }
    }

    async fn process_data(&mut self, data: &str) -> Result<()> {
        let price_histories = self.parser.parse(data)?;
        for ph in price_histories {
            self.strategy_engine.process(&ph).await?;
        }

        Ok(())
    }
}
