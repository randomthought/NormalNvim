use async_trait::async_trait;
use futures_util::future;

use crate::models::event::{Event, Market};

use crate::event::event::{EventHandler, EventProducer, Pipe};
use crate::models::event::Signal;
use crate::models::price::PriceHistory;

use super::models::order::Order;
use std::io;
use std::option::Option;
use std::sync::Arc;

pub trait TradeManager {
    fn manage(order: &Order) -> Result<(), io::Error>;
}

#[async_trait]
pub trait Algorithm {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>, io::Error>;
}

pub struct StrategyEngine {
    algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
    pipe: Arc<Box<dyn Pipe + Send + Sync>>,
}

impl StrategyEngine {
    pub fn new(
        algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
        pipe: Arc<Box<dyn Pipe + Send + Sync>>,
    ) -> Self {
        Self { algorithms, pipe }
    }
}

#[async_trait]
impl EventHandler for StrategyEngine {
    async fn handle(&self, event: Event) -> Result<(), io::Error> {
        if let Event::Market(market) = event {
            if let Market::DataEvent(data_event) = market {
                let futures: Vec<_> = self
                    .algorithms
                    .iter()
                    .map(|algo| async {
                        // TODO: Make sure you ar actually returning on a failed process error
                        if let Some(signal) = algo.process(&data_event).await? {
                            let se = Event::Signal(signal);
                            self.pipe.send(se).await?;
                        }

                        Ok(()) as Result<(), io::Error>
                    })
                    .collect();

                let _ = future::try_join_all(futures).await?;
            }
        }

        Ok(())
    }
}
