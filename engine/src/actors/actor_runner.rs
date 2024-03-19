use std::{pin::Pin, sync::Arc, time::Duration};

use crate::event_providers::provider::Parser;

use super::{
    algo_actor::AlgoActor,
    event_bus::EventBus,
    models::{AddSignalSubscribers, AlgoEventMessage},
    risk_engine_actor::RiskEngineActor,
};
use actix::Actor;
use color_eyre::eyre::Result;
use domain::{
    event::model::Event,
    risk::risk_engine::RiskEngine,
    strategy::{algorithm::Algorithm, model::algo_event::AlgoEvent},
};
use eyre::Ok;
use futures_util::{Stream, StreamExt};
use tokio::time::sleep;

pub struct ActorRunner {
    pub algorithms: Vec<Arc<dyn Algorithm>>,
    pub risk_engine: RiskEngine,
    pub parser: Arc<dyn Parser + Sync + Send>,
    pub data_stream: Pin<Box<dyn Stream<Item = Result<String>> + Sync + Send>>,
}

impl ActorRunner {
    pub async fn run(&mut self) -> Result<()> {
        let algos_addresses: Vec<_> = self
            .algorithms
            .iter()
            .map(|algo| AlgoActor {
                algorithm: algo.clone(),
                subscribers: vec![],
            })
            .map(|algo| algo.start())
            .collect();

        let risk_engine = RiskEngineActor {
            risk_engine: self.risk_engine.clone(),
            subscribers: algos_addresses.clone(),
        };

        let risk_engine_address = risk_engine.start();

        for ad in algos_addresses.iter() {
            let recipient = risk_engine_address.clone().recipient();
            let cmd = AddSignalSubscribers(recipient);
            ad.send(cmd).await?;
        }

        let event_subsribers: Vec<_> = algos_addresses
            .iter()
            .map(|addr| addr.clone().recipient())
            .collect();

        let event_bus = EventBus {
            subscribers: event_subsribers,
        };

        while let Some(dr) = self.data_stream.next().await {
            let raw_data = dr?;
            let data_event = self.parser.parse(&raw_data).await?;
            event_bus.notify(data_event)?;
        }

        Ok(())
    }
}
