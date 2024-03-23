use std::u64;

use getset::Getters;
use opentelemetry::metrics::{Counter, Histogram, Meter};

#[derive(Getters, Clone)]
pub struct Metrics {
    #[getset(get)]
    pub data_event_counter: Counter<u64>,
    #[getset(get)]
    pub algo_signal_counter: Counter<u64>,
    #[getset(get)]
    pub algo_event_counter: Counter<u64>,
    #[getset(get)]
    pub algo_histogram: Histogram<f64>,
}

impl Metrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            data_event_counter: meter
                .u64_counter("data_event_count")
                .with_description("counts number of data_events")
                .init(),
            algo_signal_counter: meter
                .u64_counter("algo_signal_count")
                .with_description("counts number of algo_signal")
                .init(),
            algo_event_counter: meter
                .u64_counter("algo_event_count")
                .with_description("counts number of algo_event")
                .init(),
            algo_histogram: meter
                .f64_histogram("on_data_elapsed_time_ms")
                .with_description("counts number of data_event")
                .init(),
        }
    }
}
