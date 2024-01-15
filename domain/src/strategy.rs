use crate::event::event::EventHandler;
use crate::event::event::EventProducer;
use crate::event::model::Event;
use crate::event::model::Market;
use crate::event::model::Signal;
use crate::models::price::PriceHistory;
use async_trait::async_trait;
use color_eyre::eyre::Ok;
use color_eyre::eyre::Result;
use futures_util::future;
use std::option::Option;
use std::sync::Arc;

#[async_trait]
pub trait Algorithm {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>>;
}

pub struct StrategyEngine {
    event_producer: Arc<dyn EventProducer + Send + Sync>,
    algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
}

impl StrategyEngine {
    pub fn new(
        algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
        event_producer: Arc<dyn EventProducer + Send + Sync>,
    ) -> Self {
        Self {
            algorithms,
            event_producer,
        }
    }

    pub async fn process(&self, price_history: &PriceHistory) -> Result<()> {
        let futures = self.algorithms.iter().map(|algo| async {
            if let Some(signal) = algo.process(price_history).await? {
                let se = Event::Signal(signal);
                self.event_producer.produce(se).await?;
            }

            Ok(()) as Result<()>
        });

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
