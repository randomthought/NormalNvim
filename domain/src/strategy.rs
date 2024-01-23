use crate::event::event::EventHandler;
use crate::event::event::EventProducer;
use crate::event::model::Event;
use crate::event::model::Market;
use crate::event::model::Signal;
use crate::models::order::Order;
use crate::models::order::OrderResult;
use crate::models::price::PriceHistory;
use async_trait::async_trait;
use color_eyre::eyre::Ok;
use color_eyre::eyre::Result;
use eyre::OptionExt;
use futures_util::future;
use std::collections::HashMap;
use std::option::Option;
use std::sync::Arc;

#[async_trait]
pub trait Algorithm {
    fn get_id(&self) -> String;
    async fn on_data(&self, price_history: &PriceHistory) -> Result<Option<Signal>>;
    async fn on_order(&self, order_result: &OrderResult) -> Result<()>;
}

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

    pub async fn process(&self, price_history: &PriceHistory) -> Result<()> {
        let futures = self.algorithms.values().map(|algo| async {
            if let Some(signal) = algo.on_data(price_history).await? {
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
            return self.process(ph).await;
        }

        match event {
            Event::Market(Market::DataEvent(ph)) => self.process(ph).await,
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
