use crate::{
    actors::actor_runner::ActorRunner,
    algorithms::fake_algo::algo::FakeAlgo,
    event_providers::back_test::BackTester,
    telemetry::{
        metrics::Metrics,
        wrappers::{algorithm::AlgorithmTelemetry, strategy_portfolio::StrategyPortfolioTelemtry},
    },
};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use color_eyre::eyre::Result;
use data_providers::{
    file,
    market::{
        forwarder::forwarder::ForwarderClient,
        polygon::{self, parser::PolygonParser},
    },
    utils,
};
use domain::{
    broker::Broker,
    risk::{algo_risk_config::AlgorithmRiskConfig, risk_engine::RiskEngine},
    strategy::algorithm::Strategy,
};
use eyre::ContextCompat;
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder, Registry, TextEncoder};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{
    env,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub async fn run_app() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let registry = prometheus::Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let provider = SdkMeterProvider::builder().with_reader(exporter).build();
    global::set_meter_provider(provider);

    let meter = global::meter("trading_engine");
    let metrics = Metrics::builder().with_meter(&meter).build()?;

    let back_tester = BackTester::new(0.05, Box::new(PolygonParser::new()));
    let back_tester_ = Arc::new(back_tester);
    let qoute_provider = back_tester_.clone();

    let broker = Broker::new(
        Decimal::from_u64(100_000).wrap_err("error parsing account balance")?,
        qoute_provider.clone(),
    );
    let broker = Arc::new(broker);

    let m = metrics.clone();
    let strategy_portfolio = StrategyPortfolioTelemtry::builder()
        .with_strategy_portfolio(broker.clone())
        .with_security_positions_gauge(m.strategy_portfolio_security_positions_gauge().clone())
        .with_security_positions_counter(m.strategy_portfolio_security_positions_counter().clone())
        .with_get_security_positions_histogram(
            m.strategy_portfolio_get_security_positions_histogram()
                .clone(),
        )
        .with_get_security_positions_error_counter(
            m.strategy_portfolio_get_security_positions_error().clone(),
        )
        .with_profit_gauge(m.strategy_portfolio_profit_gauge().clone())
        .with_get_profit_histogram(m.strategy_portfolio_get_profit_histogram().clone())
        .with_get_profit_error_counter(m.strategy_portfolio_get_profit_error_counter().clone())
        .with_pending_orders_gauge(m.strategy_portfolio_pending_orders_gauge().clone())
        .with_get_pending_histogram(m.strategy_portfolio_get_pending_histogram().clone())
        .with_get_pending_error_counter(m.strategy_portfolio_get_pending_error_counter().clone())
        .build()?;

    let strategy_portfolio = Arc::new(strategy_portfolio);

    let algos = vec![Arc::new(FakeAlgo {})];

    let algo_telems_: Result<Vec<_>, _> = algos
        .iter()
        .map(|algo| {
            let m = metrics.clone();
            AlgorithmTelemetry::builder()
                .with_algorithm(algo.clone())
                .with_strategy_id(algo.strategy_id())
                .with_event_counter(m.algorithm_event_counter().clone())
                .with_signal_counter(m.algorithm_signal_counter().clone())
                .with_histogram(m.algorithm_histogram().clone())
                .with_event_guage(m.algorithm_event_guage().clone())
                .with_on_data_error(m.algorithm_on_data_error_counter().clone())
                .build()
        })
        .collect();

    let algorithms = algo_telems_?;

    let risk_engine = algorithms
        .iter()
        .fold(Ok(&mut RiskEngine::builder()), |b_, algo| {
            let Ok(b) = b_ else {
                return b_;
            };

            AlgorithmRiskConfig::builder()
                .with_starting_balance(Decimal::new(100, 0))
                .with_strategy_id(algo.strategy_id())
                .with_max_open_trades(20)
                .build()
                .map(|conf| b.add_algorithm_risk_config(conf))
        })?
        .with_account(broker.clone())
        .with_strategy_portfolio(strategy_portfolio.clone())
        .with_order_manager(broker.clone())
        .with_qoute_provider(qoute_provider.clone())
        .build()?;

    let client = reqwest::Client::new();

    let forwarder_client = ForwarderClient::builder()
        .with_client(client.clone())
        .with_end_point("http://127.0.0.1:8081/market".into())
        .build()?;

    let data_stream = forwarder_client.get_stream().await?;

    let api_key = env::var("API_KEY")?;
    let subscription = "A.*";
    // let raw_data_stream = polygon::stream_client::create_stream(&api_key, &subscription)?;

    let file = env::var("FILE")?;
    let path = Path::new(&file);
    let buff_size = 4096usize;
    let raw_data_stream = file::utils::create_stream(path, buff_size)?;
    let parser = back_tester_.clone();

    // let data_stream = utils::parse_stream(raw_data_stream, parser.clone());
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let actor_runner = algorithms
        .into_iter()
        .fold(&mut ActorRunner::builder(), |b, x| {
            b.add_algorithm(x.strategy_id(), Arc::new(x))
        })
        .with_in_memory_broker(broker.clone())
        .with_risk_engine(risk_engine)
        .with_shutdown_signal(shutdown_signal.clone())
        .with_metrics(metrics.clone())
        .build()?;

    let metrics_server = HttpServer::new(move || {
        let registry = registry.clone();

        App::new()
            // Here, we're using an anonymous function directly
            .route(
                "/metrics",
                web::get().to(move || prometheus_metrics_api(registry.clone())),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    tokio::spawn(metrics_server);

    let runner = actor_runner.run(data_stream);
    tokio::select! {
         _ = tokio::signal::ctrl_c() => {
            println!("Shutdown signal received, shutting down...");
            shutdown_signal.store(true, Ordering::SeqCst);
        },
        Err(e) = runner => {
            println!("Server error or shutdown");
            return Err(e);
        },
    }

    Ok(())
}

async fn prometheus_metrics_api(registry: Registry) -> impl Responder {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder
        .encode(&registry.gather(), &mut buffer)
        .expect("Failed to encode metrics");

    let output = String::from_utf8(buffer).unwrap();

    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(output)
}
