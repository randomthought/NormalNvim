use color_eyre::eyre::Result;
use eyre::{bail, Context, ContextCompat, Ok};
use futures_util::future::try_join;
use std::{
    env,
    path::Path,
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
    },
};
use domain::{
    broker::broker::Broker, data::QouteProvider, portfolio::Portfolio,
    risk::risk_engine::RiskEngine,
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tokio::sync::{mpsc, Mutex};

pub async fn runApp() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let client = reqwest::Client::new();

    let back_tester = BackTester::new(0.05, Box::new(PolygonParser::new()));
    let back_tester_ = Arc::new(back_tester);
    let qoute_provider = back_tester_.clone();
    let parser = back_tester_.clone();

    let broker = Broker::new(
        Decimal::from_u64(100_000).wrap_err("error parsing account balance")?,
        qoute_provider.clone(),
    );
    let broker_ = Arc::new(broker);

    let risk_engine = RiskEngine::builder()
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
    let data_stream = file_provider::create_stream(path, buff_size)?;
    let mut actor_runner = ActorRunner {
        risk_engine,
        parser,
        data_stream,
        algorithms: vec![Arc::new(FakeAlgo {})],
    };

    actor_runner.run().await?;
    Ok(())
}
