use std::{sync::Arc, time::Instant, u64};

use async_trait::async_trait;
use derive_builder::Builder;
use domain::strategy::{
    algorithm::{Algorithm, Strategy, StrategyId},
    model::{algo_event::AlgoEvent, signal::Signal},
};
use opentelemetry::{
    metrics::{Counter, Gauge, Histogram, ObservableGauge, UpDownCounter},
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
    #[builder(setter(prefix = "with"))]
    event_guage: ObservableGauge<u64>,
    #[builder(setter(prefix = "with"))]
    on_data_error: Counter<u64>,
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
        let default_attrs = &[KeyValue::new("strategy_id", self.strategy_id)];

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
            self.histogram.record(elapsed, default_attrs);

            let Some(s) = os else {
                return result;
            };

            let attr = &[
                KeyValue::new("strategy_id", self.strategy_id),
                KeyValue::new("signal", format!("{}", s.as_ref())),
            ];

            self.signal_counter.add(1, attr);

            self.event_guage.observe(1, attr);
        } else {
            self.on_data_error.add(1, default_attrs);
        }

        result
    }
}
