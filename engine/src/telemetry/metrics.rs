use std::{time::Duration, u64};

use derive_builder::Builder;
use getset::Getters;
use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter},
};
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        PeriodicReader, SdkMeterProvider,
    },
    runtime,
};
use prometheus::{core::MetricVecBuilder, proto::MetricFamily, Encoder, TextEncoder};

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

        self
    }
}

pub fn init_otel() {
    // let export_config = ExportConfig {
    //     endpoint: "http://localhost:4317".to_string(),
    //     timeout: Duration::from_secs(3),
    //     protocol: Protocol::Grpc,
    // };

    // let provider = opentelemetry_otlp::new_pipeline()
    //     .metrics(opentelemetry_sdk::runtime::Tokio)
    //     .with_exporter(
    //         opentelemetry_otlp::new_exporter()
    //             .tonic()
    //             .with_endpoint("http://localhost:4317"),
    //     )
    //     .with_period(Duration::from_secs(3))
    //     .with_timeout(Duration::from_secs(10))
    //     .with_aggregation_selector(DefaultAggregationSelector::new())
    //     .with_temporality_selector(DefaultTemporalitySelector::new())
    //     .build()
    //     .unwrap();

    // let exporter = opentelemetry_stdout::MetricsExporter::default();
    // let exporter = opentelemetry_stdout::MetricsExporterBuilder::default()
    //     .with_encoder(|writer, data| Ok(serde_json::to_writer_pretty(writer, &data).unwrap()))
    //     .build();
    // let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    // let provider = SdkMeterProvider::builder().with_reader(reader).build();
    // global::set_meter_provider(provider);

    let registry = prometheus::Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let provider = SdkMeterProvider::builder().with_reader(exporter).build();
    global::set_meter_provider(provider);
}
