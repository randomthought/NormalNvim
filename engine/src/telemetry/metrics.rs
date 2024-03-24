use std::u64;

use derive_builder::Builder;
use getset::Getters;
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter, ObservableGauge};

#[derive(Builder, Getters, Clone)]
pub struct Metrics {
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub data_event_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub algo_signal_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub algo_event_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub algo_histogram: Histogram<f64>,
    #[builder(setter(prefix = "with"))]
    pub algo_event_guage: ObservableGauge<u64>,
}

impl Metrics {
    pub fn builder() -> MetricsBuilder {
        MetricsBuilder::default()
    }
}

impl MetricsBuilder {
    pub fn with_meter(&mut self, value: &Meter) -> &mut Self {
        self.data_event_counter = Some(
            value
                .u64_counter("data_event_count")
                .with_description("counts number of data_events")
                .init(),
        );
        self.algo_signal_counter = Some(
            value
                .u64_counter("algo_signal_count")
                .with_description("counts number of algo_signal")
                .init(),
        );
        self.algo_event_counter = Some(
            value
                .u64_counter("algo_event_count")
                .with_description("counts number of algo_event")
                .init(),
        );
        self.algo_histogram = Some(
            value
                .f64_histogram("on_data_elapsed_time_ms")
                .with_description("counts number of data_event")
                .init(),
        );

        self.algo_event_guage = Some(
            value
                .u64_observable_gauge("algo_event_guage")
                .with_description("records algo_event")
                .init(),
        );

        self
    }
}
