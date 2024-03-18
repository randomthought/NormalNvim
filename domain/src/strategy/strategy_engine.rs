use crate::event::event::EventHandler;
use crate::event::event::EventProducer;
use crate::event::model::Event;
use crate::event::model::Market;
use crate::models::orders::pending_order::Order;
use async_trait::async_trait;
use futures_util::future;
use std::collections::HashMap;
use std::sync::Arc;

use super::algorithm::Algorithm;
use super::algorithm::StrategyId;
use super::model::algo_event::AlgoEvent;

pub struct StrategyEngine {
    event_producer: Arc<dyn EventProducer + Send + Sync>,
    algorithms: HashMap<StrategyId, Box<dyn Algorithm + Send + Sync>>,
}

impl StrategyEngine {
    pub fn new(
        algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
        event_producer: Arc<dyn EventProducer + Send + Sync>,
    ) -> Self {
        let mut map = HashMap::new();
        for algo in algorithms {
            map.insert(algo.strategy_id(), algo);
        }
        Self {
            algorithms: map,
            event_producer,
        }
    }

    pub async fn process(&self, market: &Market) -> Result<(), crate::error::Error> {
        let futures = self.algorithms.values().map(|algo| async {
            let algo_event = AlgoEvent::Market(market);
            if let Some(signal) = algo.on_event(algo_event).await? {
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

                let algo =
                    self.algorithms
                        .get(ao.startegy_id())
                        .ok_or(crate::error::Error::Message(
                            "unable to find algorithm".to_string(),
                        ))?;

                let algo_event = AlgoEvent::OrderResult(&or);
                let _ = algo.on_event(algo_event).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
