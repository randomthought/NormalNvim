use crate::event::event::EventHandler;
use crate::event::event::EventProducer;
use crate::event::model::Event;
use crate::event::model::Market;
use crate::models::order::Order;
use async_trait::async_trait;
use color_eyre::eyre::Ok;
use color_eyre::eyre::Result;
use eyre::OptionExt;
use futures_util::future;
use std::collections::HashMap;
use std::sync::Arc;

use super::algo_event::AlgoEvent;
use super::algorithm::Algorithm;
use super::algorithm::StrategyId;

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

    pub async fn process(&self, market: &Market) -> Result<()> {
        let futures = self.algorithms.values().map(|algo| async {
            let algo_event = AlgoEvent::Market(market);
            if let Some(signal) = algo.on_event(algo_event).await? {
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
        match event {
            Event::Market(m) => self.process(m).await,
            Event::Order(ao) => {
                let Order::OrderResult(or) = ao else {
                    return Ok(());
                };

                let algo = self
                    .algorithms
                    .get(ao.startegy_id())
                    .ok_or_eyre("unable to find algorithm")?;

                let algo_event = AlgoEvent::OrderResult(&or);
                let _ = algo.on_event(algo_event).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
