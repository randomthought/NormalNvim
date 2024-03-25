use std::{sync::Arc, u64};

use derive_builder::Builder;
use getset::Getters;
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter, ObservableGauge};

#[derive(Builder, Getters, Clone)]
pub struct Metrics {
    // Algorithm Metrics
    // =======================================
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub data_event_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub algorithm_signal_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub algorithm_event_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub algorithm_histogram: Histogram<f64>,
    #[builder(setter(prefix = "with"))]
    pub algorithm_event_guage: ObservableGauge<u64>,
    #[builder(setter(prefix = "with"))]
    pub algorithm_on_data_error_counter: Counter<u64>,
    // Strategy Portfolio Metrics
    // =======================================
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_security_positions_gauge: ObservableGauge<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_security_positions_counter: Counter<u64>,
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_security_positions_error_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_profit_gauge: ObservableGauge<f64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_get_profit_histogram: Histogram<f64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_get_profit_error_counter: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_security_positions_guage: ObservableGauge<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_get_security_positions_histogram: Histogram<f64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_get_security_positions_error: Counter<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_pending_orders_gauge: ObservableGauge<u64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_get_pending_histogram: Histogram<f64>,
    #[getset(get)]
    #[builder(setter(prefix = "with"))]
    pub strategy_portfolio_get_pending_error_counter: Counter<u64>,
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
        self.algorithm_signal_counter = Some(
            value
                .u64_counter("algorithm_signal_counter")
                .with_description("counts number of algo_signal")
                .init(),
        );
        self.algorithm_event_counter = Some(
            value
                .u64_counter("algo_event_count")
                .with_description("counts number of algo_event")
                .init(),
        );
        self.algorithm_histogram = Some(
            value
                .f64_histogram("on_data_elapsed_time_ms")
                .with_description("counts number of data_event")
                .init(),
        );

        self.algorithm_event_guage = Some(
            value
                .u64_observable_gauge("algo_event_guage")
                .with_description("records algo_event")
                .init(),
        );

        self.algorithm_on_data_error_counter = Some(
            value
                .u64_counter("algorithm_on_data_error_counter")
                .with_description("records algo_event")
                .init(),
        );

        self.strategy_portfolio_security_positions_gauge = Some(
            value
                .u64_observable_gauge("strategy_portfolio_security_positions_gauge")
                .init(),
        );

        self.strategy_portfolio_security_positions_counter = Some(
            value
                .u64_counter("strategy_portfolio_security_positions_counter")
                .init(),
        );

        self.strategy_portfolio_security_positions_error_counter = Some(
            value
                .u64_counter("strategy_portfolio_security_positions_error_counter")
                .init(),
        );

        self.strategy_portfolio_profit_gauge = Some(
            value
                .f64_observable_gauge("strategy_portfolio_profit_gauge")
                .init(),
        );

        self.strategy_portfolio_get_profit_histogram = Some(
            value
                .f64_histogram("strategy_portfolio_get_profit_histogram")
                .init(),
        );

        self.strategy_portfolio_get_profit_error_counter = Some(
            value
                .u64_counter("strategy_portfolio_get_profit_error_counter")
                .init(),
        );

        self.strategy_portfolio_security_positions_guage = Some(
            value
                .u64_observable_gauge("strategy_portfolio_security_positions_guage")
                .init(),
        );

        self.strategy_portfolio_get_security_positions_histogram = Some(
            value
                .f64_histogram("strategy_portfolio_get_security_positions_histogram")
                .init(),
        );

        self.strategy_portfolio_get_security_positions_error = Some(
            value
                .u64_counter("strategy_portfolio_get_security_positions_error")
                .init(),
        );

        self.strategy_portfolio_pending_orders_gauge = Some(
            value
                .u64_observable_gauge("strategy_portfolio_pending_orders_gauge")
                .init(),
        );

        self.strategy_portfolio_get_pending_histogram = Some(
            value
                .f64_histogram("strategy_portfolio_get_pending_histogram")
                .init(),
        );

        self.strategy_portfolio_get_pending_error_counter = Some(
            value
                .u64_counter("strategy_portfolio_get_pending_error_counter")
                .init(),
        );

        self
    }
}
