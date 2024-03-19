use std::{pin::Pin, sync::Arc};

use crate::event_providers::provider::Parser;

use super::{
    algo_actor::AlgoActor, event_bus::EventBus, models::AlgoEventMessage,
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

pub struct ActorRunner {
    algorithms: Vec<Arc<dyn Algorithm>>,
    risk_engine: RiskEngine,
    // event_bus: EventBus,
    parser: Box<dyn Parser + Sync + Send>,
}

impl ActorRunner {
    pub async fn run(
        &self,
        mut data_stream: Pin<Box<dyn Stream<Item = Result<String>> + Sync + Send>>,
    ) -> Result<()> {
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

        let _ = risk_engine.start();

        let event_subsribers: Vec<_> = algos_addresses
            .iter()
            .map(|addr| addr.clone().recipient())
            .collect();

        let event_bus = EventBus {
            subscribers: event_subsribers,
        };

        while let Some(dr) = data_stream.next().await {
            let raw_data = dr?;
            let event = self.parser.parse(&raw_data).await?;
            if let Event::Market(x) = event {
                let algo_msg = AlgoEvent::Market(x);
                event_bus.notify(algo_msg);
            }
        }

        Ok(())
    }
}
