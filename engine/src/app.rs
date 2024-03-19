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
    algorithms::fake_algo::FakeAlgo,
    event_providers::{
        back_test::BackTester,
        file_provider,
        market::polygon::{api_client, parser::PolygonParser},
        utils::EventStream,
    },
};
use domain::{
    broker::broker::Broker,
    data::QouteProvider,
    event::{self, event::EventHandler},
    portfolio::Portfolio,
    risk::{config::RiskEngineConfig, risk_engine::RiskEngine},
    runner::Runner,
    strategy::{algorithm::Algorithm, strategy::Strategy, strategy_engine::StrategyEngine},
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tokio::sync::{mpsc, Mutex};

pub async fn runApp() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let client = reqwest::Client::new();
    let api_key = env::var("API_KEY")?;

    let back_tester = BackTester::new(0.05, Box::new(PolygonParser::new()));
    let back_tester_ = Arc::new(back_tester);
    let qoute_provider = back_tester_.clone();
    let parser = back_tester_.clone();
    let (sender, reciever) = mpsc::channel(2);
    let channel_producer = event::channel::ChannelProducer::new(sender);
    let event_producer = Arc::new(channel_producer);

    let broker = Broker::new(
        Decimal::from_u64(100_000).wrap_err("error parsing account balance")?,
        qoute_provider.clone(),
        event_producer.clone(),
    );
    let broker_ = Arc::new(broker);
    let risk_engine_config = RiskEngineConfig {
        max_trade_portfolio_accumulaton: 0.10,
        max_portfolio_risk: 0.10,
        max_open_trades: None,
    };

    let portfolio = Portfolio::new(broker_.clone(), broker_.clone(), qoute_provider.clone());
    let risk_egnine = RiskEngine::new(
        risk_engine_config,
        broker_.clone(),
        event_producer.clone(),
        qoute_provider.clone(),
        broker_.clone(),
        Box::new(portfolio),
    );

    let strategy = Strategy::builder()
        .with_algorithm(Box::new(FakeAlgo {}))
        .with_portfolio(broker_.clone())
        .with_qoute_provider(qoute_provider.clone())
        .with_open_trades(4)
        .build()
        .unwrap();

    let strategies = vec![strategy];
    let strategy_engine = StrategyEngine::new(strategies, event_producer.clone());

    let event_handlers: Vec<Box<dyn EventHandler + Sync + Send>> =
        vec![Box::new(strategy_engine), Box::new(risk_egnine)];

    let t1 = async move {
        let subscription = "A.*";

        // let data_stream = engine::event_providers::market::polygon::stream_client::create_stream(
        //     &api_key,
        //     &subscription,
        // )
        // ?;

        println!("calling t1");
        let file = env::var("FILE")?;
        let path = Path::new(&file);
        let buff_size = 4096usize;
        let data_stream = file_provider::create_stream(path, buff_size)?;

        let mut event_stream =
            EventStream::new(event_producer.clone(), data_stream, parser.clone());

        event_stream.start().await
    };

    let t2 = async move {
        println!("calling t2");
        let rs = ReceiverStream::new(reciever);
        let stream = Box::pin(rs);
        let mut event_runner = Runner::new(event_handlers, stream);
        event_runner.run().await
    };

    // TODO: findout how to 'race' threads or stop all thereads on the first one to finish
    // tokio::spawn(t1).await?;
    // tokio::spawn(t2).await?;

    tokio::join!(t1, t2);

    Ok(())
}