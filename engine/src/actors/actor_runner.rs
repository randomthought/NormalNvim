use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::telemetry::metrics::Metrics;

use super::{
    algo_actor::AlgoActor, broker_price_event_actor::BrokerPriceEventActor, event_bus::EventBus,
    models::AddSignalSubscribers, risk_engine_actor::RiskEngineActor,
};
use actix::Actor;
use derive_builder::Builder;
use domain::{broker::Broker, risk::risk_engine::RiskEngine};
use futures_util::{Stream, StreamExt};
use models::{event::DataEvent, strategy::common::StrategyId};
use traits::strategy::algorithm::Algorithm;

#[derive(Builder)]
#[builder(public, setter(prefix = "with", strip_option))]
pub struct ActorRunner {
    #[builder(private)]
    algorithms: Vec<(StrategyId, Arc<dyn Algorithm + Send + Sync>)>,
    risk_engine: RiskEngine,
    #[builder(default)]
    in_memory_broker: Option<Arc<Broker>>,
    shutdown_signal: Arc<AtomicBool>,
    metrics: Metrics,
}

impl ActorRunner {
    pub fn builder() -> ActorRunnerBuilder {
        ActorRunnerBuilder::default()
    }

    pub async fn run(
        &self,
        mut data_stream: Pin<Box<dyn Stream<Item = eyre::Result<Option<DataEvent>>> + Send>>,
    ) -> eyre::Result<()> {
        let algos_addresses: Result<Vec<_>, _> = self
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
        let algos_addresses = algos_addresses?;

        let metrics = self.metrics.clone();
        let risk_engine = algos_addresses
            .clone()
            .into_iter()
            .fold(&mut RiskEngineActor::builder(), |b, (id, algo_add)| {
                b.add_subscriber(id, algo_add)
            })
            .with_risk_engine(self.risk_engine.clone())
            .with_risk_engine_error_counter(metrics.risk_engine_error_counter().clone())
            .with_risk_engine_order_result_gauge(metrics.risk_engine_order_result_gauge().clone())
            .with_risk_engine_order_result_counter(
                metrics.risk_engine_order_result_counter().clone(),
            )
            .with_risk_engine_process_signal_histogram(
                metrics.risk_engine_process_signal_histogram().clone(),
            )
            .build()?;

        let risk_engine_address = risk_engine.start();

        for (_, ad) in algos_addresses.iter() {
            let recipient = risk_engine_address.clone().recipient();
            let cmd = AddSignalSubscribers(recipient);
            ad.send(cmd).await?;
        }

        let mut event_subsribers: Vec<_> = algos_addresses
            .iter()
            .map(|(_, addr)| addr.clone().recipient())
            .collect();

        if let Some(broker) = self.in_memory_broker.clone() {
            let broker_price_event_actor = algos_addresses
                .iter()
                .fold(
                    &mut BrokerPriceEventActor::builder(),
                    |b, (id, algo_add)| b.add_subscriber(id, algo_add.clone()),
                )
                .with_in_memory_broker(broker)
                .build()?;

            let addr = broker_price_event_actor.start();
            let rec = addr.recipient();

            event_subsribers.push(rec);
        }

        let event_bus = event_subsribers
            .into_iter()
            .fold(&mut EventBus::builder(), |b, ebs| b.add_subscriber(ebs))
            .build()?;

        while let Some(dr) = data_stream.next().await {
            if self.shutdown_signal.load(Ordering::SeqCst) {
                break;
            }
            if let Some(data_event) = dr? {
                event_bus.notify(data_event)?;
            }
        }

        Ok(())
    }
}

impl ActorRunnerBuilder {
    pub fn add_algorithm(
        &mut self,
        strategy_id: StrategyId,
        algorithm: Arc<dyn Algorithm + Send + Sync>,
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
