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

use super::algorithm::Algorithm;

pub struct StrategyEngine {
    event_producer: Arc<dyn EventProducer + Send + Sync>,
    algorithms: HashMap<String, Box<dyn Algorithm + Send + Sync>>,
}

impl StrategyEngine {
    pub fn new(
        algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
        event_producer: Arc<dyn EventProducer + Send + Sync>,
    ) -> Self {
        let mut map = HashMap::new();
        for algo in algorithms {
            map.insert(algo.get_id(), algo);
        }
        Self {
            algorithms: map,
            event_producer,
        }
    }

    pub async fn process(&self, market: &Market) -> Result<()> {
        let futures = self.algorithms.values().map(|algo| async {
            if let Some(signal) = algo.on_data(market).await? {
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
            Event::AlgoOrder(ao) => {
                let Order::OrderResult(or) = ao.order.clone() else {
                    return Ok(());
                };

                let algo = self
                    .algorithms
                    .get(&ao.strategy_id)
                    .ok_or_eyre("enable to find algorithm")?;

                algo.on_order(&or).await
            }
            _ => Ok(()),
        }
    }
}
