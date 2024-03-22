use color_eyre::eyre::Result;
use eyre::{bail, Context, ContextCompat};
use futures_util::future::try_join;
use std::{
    env,
    path::Path,
    pin::Pin,
    sync::{atomic::AtomicBool, Arc},
};
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    actors::actor_runner::ActorRunner,
    algorithms::fake_algo::algo::FakeAlgo,
    event_providers::{
        back_test::BackTester,
        file_provider,
        market::polygon::{self, api_client, parser::PolygonParser},
        provider::Parser,
        utils,
    },
};
use domain::{
    broker::broker::Broker,
    data::QouteProvider,
    portfolio::Portfolio,
    risk::{algo_risk_config::AlgorithmRiskConfig, risk_engine::RiskEngine},
    strategy::algorithm::Strategy,
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tokio::sync::{mpsc, Mutex};

pub async fn runApp() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let client = reqwest::Client::new();

    let back_tester = BackTester::new(0.05, Box::new(PolygonParser::new()));
    let back_tester_ = Arc::new(back_tester);
    let qoute_provider = back_tester_.clone();

    let broker = Broker::new(
        Decimal::from_u64(100_000).wrap_err("error parsing account balance")?,
        qoute_provider.clone(),
    );
    let broker_ = Arc::new(broker);

    let algorithms = vec![Arc::new(FakeAlgo {})];

    let risk_engine = algorithms
        .iter()
        .fold(Ok(&mut RiskEngine::builder()), |b_, algo| {
            let Ok(b) = b_ else {
                return b_;
            };

            AlgorithmRiskConfig::builder()
                .with_strategy_id(algo.strategy_id())
                .with_max_open_trades(2)
                .build()
                .map(|conf| b.add_algorithm_risk_config(conf))
        })?
        .with_strategy_portrfolio(broker_.clone())
        .with_order_manager(broker_.clone())
        .with_qoute_provider(qoute_provider.clone())
        .build()?;

    // let api_key = env::var("API_KEY")?;
    // let subscription = "A.*";
    // let data_stream = polygon::stream_client::create_stream(&api_key, &subscription)?;

    let file = env::var("FILE")?;
    let path = Path::new(&file);
    let buff_size = 4096usize;
    let file_stream = file_provider::create_stream(path, buff_size)?;
    let parser = back_tester_.clone();
    let data_stream = utils::parse_stream(file_stream, parser.clone());
    // let mut actor_runner = ActorRunner {
    //     risk_engine,
    //     parser,
    //     data_stream,
    //     algorithms: vec![Arc::new(FakeAlgo {})],
    // };
    let actor_runner = algorithms
        .into_iter()
        .fold(&mut ActorRunner::builder(), |b, x| {
            b.add_algorithm(x.strategy_id(), x)
        })
        .with_parser(parser.clone())
        .with_risk_engine(risk_engine)
        .build()?;

    actor_runner.run(data_stream).await?;

    Ok(())
}
