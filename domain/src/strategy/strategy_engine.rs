use crate::event::event::EventHandler;
use crate::event::event::EventProducer;
use crate::event::model::Event;
use crate::event::model::Market;
use crate::models::orders::pending_order::Order;
use async_trait::async_trait;
use futures_util::future;
use std::sync::Arc;

use super::model::algo_event::AlgoEvent;
use super::strategy::Strategy;

pub struct StrategyEngine {
    event_producer: Arc<dyn EventProducer + Send + Sync>,
    strategies: Vec<Strategy>,
}

impl StrategyEngine {
    pub fn new(
        strategies: Vec<Strategy>,
        event_producer: Arc<dyn EventProducer + Send + Sync>,
    ) -> Self {
        Self {
            strategies,
            event_producer,
        }
    }

    pub async fn process(&self, market: &Market) -> Result<(), crate::error::Error> {
        let futures = self.strategies.iter().map(|algo| async {
            let algo_event = AlgoEvent::Market(market);
            if let Some(signal) = algo
                .on_event(algo_event)
                .await
                .map_err(|e| crate::error::Error::Any(e.into()))?
            {
                let se = Event::Signal(signal);
                self.event_producer.produce(se).await?;
            }

            Ok(()) as Result<(), crate::error::Error>
        });

        let _ = future::try_join_all(futures).await?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for StrategyEngine {
    async fn handle(&self, event: &Event) -> Result<(), crate::error::Error> {
        match event {
            Event::Market(m) => self.process(m).await,
            Event::Order(ao) => {
                let Order::OrderResult(or) = ao else {
                    return Ok(());
                };

                let futures = self.strategies.iter().map(|algo| async {
                    let algo_event = AlgoEvent::OrderResult(&or);
                    if let Some(signal) = algo
                        .on_event(algo_event)
                        .await
                        .map_err(|e| crate::error::Error::Any(e.into()))?
                    {
                        let se = Event::Signal(signal);
                        self.event_producer.produce(se).await?;
                    }

                    Ok(()) as Result<(), crate::error::Error>
                });

                let _ = future::try_join_all(futures).await?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
