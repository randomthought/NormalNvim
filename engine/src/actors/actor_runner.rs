use std::{pin::Pin, sync::Arc, time::Duration};

use crate::event_providers::provider::Parser;

use super::{
    algo_actor::AlgoActor, event_bus::EventBus, models::AddSignalSubscribers,
    risk_engine_actor::RiskEngineActor,
};
use actix::Actor;
use derive_builder::Builder;
use domain::{
    event::model::DataEvent,
    risk::risk_engine::RiskEngine,
    strategy::algorithm::{Algorithm, Strategy, StrategyId},
};
use futures_util::{Stream, StreamExt};

#[derive(Builder)]
pub struct ActorRunner {
    #[builder(private)]
    algorithms: Vec<(StrategyId, Arc<dyn Algorithm>)>,
    #[builder(public, setter(prefix = "with"))]
    risk_engine: RiskEngine,
}

impl ActorRunner {
    pub fn builder() -> ActorRunnerBuilder {
        ActorRunnerBuilder::default()
    }

    pub async fn run(
        &self,
        mut data_stream: Pin<Box<dyn Stream<Item = eyre::Result<DataEvent>> + Send>>,
    ) -> eyre::Result<()> {
        let algos_addresses_: Result<Vec<_>, _> = self
            .algorithms
            .clone()
            .into_iter()
            .map(|(id, algo)| {
                let ao = AlgoActor::builder().with_algorithm(algo).build();
                match ao {
                    Ok(x) => Ok((id, x.start())),
                    Err(x) => Err(x),
                }
            })
            .collect();
        let algos_addresses = algos_addresses_?;

        let risk_engine = algos_addresses
            .clone()
            .into_iter()
            .fold(&mut RiskEngineActor::builder(), |b, (id, algo_add)| {
                b.add_subscriber(id, algo_add)
            })
            .with_risk_engine(self.risk_engine.clone())
            .build()?;

        let risk_engine_address = risk_engine.start();

        for (_, ad) in algos_addresses.iter() {
            let recipient = risk_engine_address.clone().recipient();
            let cmd = AddSignalSubscribers(recipient);
            ad.send(cmd).await?;
        }

        let event_subsribers: Vec<_> = algos_addresses
            .iter()
            .map(|(_, addr)| addr.clone().recipient())
            .collect();

        let event_bus = event_subsribers
            .into_iter()
            .fold(&mut EventBus::builder(), |b, ebs| b.add_subscriber(ebs))
            .build()?;

        while let Some(dr) = data_stream.next().await {
            event_bus.notify(dr?)?;
        }

        Ok(())
    }
}

impl ActorRunnerBuilder {
    pub fn add_algorithm(
        &mut self,
        strategy_id: StrategyId,
        algorithm: Arc<dyn Algorithm>,
    ) -> &mut Self {
        let entry = (strategy_id, algorithm);

        if let Some(algorithms) = self.algorithms.as_mut() {
            algorithms.push(entry);
            return self;
        }

        self.algorithms = Some(vec![entry]);

        self
    }
}
