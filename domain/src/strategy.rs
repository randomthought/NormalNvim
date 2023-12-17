use crate::event::event::EventHandler;
use crate::event::event::EventProducer;
use crate::event::model::Event;
use crate::event::model::Market;
use crate::event::model::Signal;
use crate::models::price::PriceHistory;
use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::future;
use std::option::Option;

#[async_trait]
pub trait Algorithm {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>>;
}

pub struct StrategyEngine {
    event_producer: Box<dyn EventProducer + Send + Sync>,
    algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
}

impl StrategyEngine {
    pub fn new(
        algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
        event_producer: Box<dyn EventProducer + Send + Sync>,
    ) -> Self {
        Self {
            algorithms,
            event_producer,
        }
    }

    pub async fn process(&self, price_history: &PriceHistory) -> Result<()> {
        let futures: Vec<_> = self
            .algorithms
            .iter()
            .map(|algo| async {
                if let Some(signal) = algo.process(price_history).await? {
                    let se = Event::Signal(signal);
                    self.event_producer.produce(se).await?;
                }

                Ok(()) as Result<()>
            })
            .collect();

        let _ = future::try_join_all(futures).await?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for StrategyEngine {
    async fn handle(&self, event: &Event) -> Result<()> {
        if let Event::Market(Market::DataEvent(ph)) = event {
            self.process(ph).await?;
        }

        Ok(())
    }
}
