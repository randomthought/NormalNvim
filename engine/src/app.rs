use color_eyre::eyre::Result;
use eyre::{bail, Context, ContextCompat, Ok};
use futures_util::future::try_join;
use std::{
    env,
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};

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
    strategy::{algorithm::Algorithm, strategy_engine::StrategyEngine},
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tokio::sync::Mutex;

pub async fn runApp() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let client = reqwest::Client::new();
    let api_key = env::var("API_KEY")?;

    let back_tester = BackTester::new(0.05, Box::new(PolygonParser::new()));
    let back_tester_ = Arc::new(back_tester);
    let qoute_provider = back_tester_.clone();
    let parser = back_tester_.clone();

    let event_channel = event::channel::Channel::new();
    let event_channel_ = Arc::new(event_channel.clone());

    let broker = Broker::new(
        Decimal::from_u64(100_000).wrap_err("error parsing account balance")?,
        qoute_provider.clone(),
        event_channel_.clone(),
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
        event_channel_.clone(),
        qoute_provider.clone(),
        broker_.clone(),
        Box::new(portfolio),
    );

    let algorithms: Vec<Box<dyn Algorithm + Send + Sync>> = vec![Box::new(FakeAlgo {})];
    let strategy_engine = StrategyEngine::new(algorithms, event_channel_.clone());

    let event_handlers: Vec<Box<dyn EventHandler + Sync + Send>> =
        vec![Box::new(strategy_engine), Box::new(risk_egnine)];

    let exit_signal = Arc::new(AtomicBool::from(false));
    let exit_signal_t1 = exit_signal.clone();
    let exit_signal_t2 = exit_signal.clone();
    let t1 = async move {
        let subscription = "A.*";

        // let data_stream = engine::event_providers::market::polygon::stream_client::create_stream(
        //     &api_key,
        //     &subscription,
        // )
        // ?;

        let file = env::var("FILE")?;
        let path = Path::new(&file);
        let data_stream = file_provider::create_stream(path)?;

        let mut event_stream = EventStream::new(
            event_channel_.clone(),
            data_stream,
            parser.clone(),
            exit_signal_t1,
        );

        event_stream.start().await
    };

    let t2 = async move {
        let stream = Box::pin(event_channel.clone());
        let mut event_runner = Runner::new(event_handlers, stream, exit_signal_t2);
        event_runner.run().await
    };

    // TODO: findout how to 'race' threads or stop all thereads on the first one to finish
    tokio::spawn(t1).await?;
    tokio::spawn(t2).await?;

    Ok(())
}
