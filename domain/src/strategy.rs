use async_trait::async_trait;
use futures_util::future;

use crate::models::event::{Event, Market};

use crate::event::event::{EventHandler, EventProducer};
use crate::models::event::Signal;
use crate::models::price::PriceHistory;

use super::models::order::Order;
use std::io;
use std::option::Option;

pub trait TradeManager {
    fn manage(order: &Order) -> Result<(), io::Error>;
}

#[async_trait]
pub trait Algorithm: Sync + Send {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>, io::Error>;
}

pub struct StrategyEngine<'a> {
    algorithms: &'a [&'a dyn Algorithm],
    event_producer: &'a dyn EventProducer,
}

impl<'a> StrategyEngine<'a> {
    pub fn new(
        &self,
        algorithms: &'a [&'a dyn Algorithm],
        event_producer: &'a dyn EventProducer,
    ) -> Self {
        Self {
            algorithms,
            event_producer,
        }
    }
}

#[async_trait]
impl<'a> EventHandler for StrategyEngine<'a> {
    async fn handle(&self, event: &Event) -> Result<(), io::Error> {
        if let Event::Market(market) = event {
            if let Market::DataEvent(data_event) = market {
                let futures: Vec<_> = self
                    .algorithms
                    .iter()
                    .map(|algo| async {
                        // TODO: Make sure you ar actually returning on a failed process error
                        if let Some(signal) = algo.process(data_event).await? {
                            let se = Event::Signal(signal);
                            return self.event_producer.produce(&se).await;
                        }

                        Ok(())
                    })
                    .collect();

                let _ = future::try_join_all(futures).await?;
            }
        }

        Ok(())
    }
}
