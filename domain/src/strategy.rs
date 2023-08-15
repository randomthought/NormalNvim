use async_trait::async_trait;
use futures_util::future;

use crate::models::event::Event;

use crate::models::event::Signal;
use crate::models::price::PriceHistory;
use crate::risk::RiskEngine;

use std::io;
use std::option::Option;

#[async_trait]
pub trait Algorithm {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>, io::Error>;
}

pub struct StrategyEngine {
    risk_egnine: Box<RiskEngine>,
    algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
}

impl StrategyEngine {
    pub fn new(
        risk_egnine: Box<RiskEngine>,
        algorithms: Vec<Box<dyn Algorithm + Send + Sync>>,
    ) -> Self {
        Self {
            risk_egnine,
            algorithms,
        }
    }

    pub async fn process(&self, price_data: PriceHistory) -> Result<(), io::Error> {
        let futures: Vec<_> = self
            .algorithms
            .iter()
            .map(|algo| async {
                // TODO: Make sure you ar actually returning on a failed process error
                if let Some(signal) = algo.process(&price_data).await? {
                    self.risk_egnine.process_signal(signal.clone()).await?;

                    // TODO: add way to send signals to some stream
                    let se = Event::Signal(signal);
                }

                Ok(()) as Result<(), io::Error>
            })
            .collect();

        let _ = future::try_join_all(futures).await?;

        Ok(())
    }
}
