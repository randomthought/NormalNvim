use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use derive_builder::Builder;
use domain::strategy::{
    algorithm::{Algorithm, Strategy, StrategyId},
    model::{algo_event::AlgoEvent, signal::Signal},
};
use opentelemetry::{
    metrics::{Counter, Histogram},
    KeyValue,
};

#[derive(Builder, Clone)]
pub struct AlgorithmTelemetry {
    #[builder(setter(prefix = "with"))]
    strategy_id: StrategyId,
    #[builder(setter(prefix = "with"))]
    algorithm: Arc<dyn Algorithm + Sync + Send>,
    #[builder(setter(prefix = "with"))]
    signal_counter: Counter<u64>,
    #[builder(setter(prefix = "with"))]
    event_counter: Counter<u64>,
    #[builder(setter(prefix = "with"))]
    histogram: Histogram<f64>,
}

impl AlgorithmTelemetry {
    pub fn builder() -> AlgorithmTelemetryBuilder {
        AlgorithmTelemetryBuilder::default()
    }
}

impl Strategy for AlgorithmTelemetry {
    fn strategy_id(&self) -> StrategyId {
        self.strategy_id
    }
}

#[async_trait]
impl Algorithm for AlgorithmTelemetry {
    async fn on_event(
        &self,
        algo_event: AlgoEvent,
    ) -> Result<Option<Signal>, domain::error::Error> {
        self.event_counter.add(
            1,
            &[
                KeyValue::new("strategy_id", self.strategy_id),
                KeyValue::new("algo_event", format!("{}", algo_event.as_ref())),
            ],
        );

        let start_time = Instant::now();
        let result = self.algorithm.on_event(algo_event).await;

        if let Ok(os) = result.as_ref() {
            let elapsed = start_time.elapsed().as_millis() as f64;
            self.histogram
                .record(elapsed, &[KeyValue::new("strategy_id", self.strategy_id)]);

            let Some(s) = os else {
                return result;
            };

            self.signal_counter.add(
                1,
                &[
                    KeyValue::new("strategy_id", self.strategy_id),
                    KeyValue::new("signal", format!("{}", s.as_ref())),
                ],
            );
        }

        result
    }
}
