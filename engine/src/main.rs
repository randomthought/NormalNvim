use std::{env, sync::Arc};

use domain::{
    broker::Broker,
    event::{self, event::EventHandler},
    portfolio::Portfolio,
    risk::{config::RiskEngineConfig, risk_engine::RiskEngine},
    runner::Runner,
    strategy::{Algorithm, StrategyEngine},
};
use engine::{
    algorithms::fake_algo::FakeAlgo,
    event_providers::{
        file_provider,
        market::polygon::{api_client, parser::PolygonParser},
        utils::EventStream,
    },
};
use rust_decimal::{prelude::FromPrimitive, Decimal};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let api_key = env::var("API_KEY").unwrap();
    let quite_provider = api_client::ApiClient::new(api_key.clone(), client);
    let quite_provider_ = Arc::new(quite_provider);

    let event_channel = event::channel::Channel::new();
    let event_channel_ = Arc::new(event_channel.clone());

    let broker = Broker::new(Decimal::from_u64(100_000).unwrap(), event_channel_.clone());
    let broker_ = Arc::new(broker);
    let risk_engine_config = RiskEngineConfig {
        max_trade_portfolio_accumulaton: 0.10,
        max_portfolio_risk: 0.10,
        max_risk_per_trade: 0.005,
        max_open_trades: None,
    };

    let portfolio = Portfolio::new(broker_.clone(), broker_.clone(), quite_provider_.clone());
    let risk_egnine = RiskEngine::new(
        risk_engine_config,
        quite_provider_.clone(),
        broker_.clone(),
        Box::new(portfolio),
    );

    let algorithms: Vec<Box<dyn Algorithm + Send + Sync>> = vec![Box::new(FakeAlgo {})];
    let strategy_engine = StrategyEngine::new(algorithms, event_channel_.clone());

    let event_handlers: Vec<Box<dyn EventHandler + Sync + Send>> =
        vec![Box::new(strategy_engine), Box::new(risk_egnine)];

    let t1 = async {
        let subscription = "A.*";
        let data_stream = engine::event_providers::market::polygon::stream_client::create_stream(
            &api_key,
            &subscription,
        )
        .unwrap();
        let parser = Box::new(PolygonParser::new());

        // let path = Path::new("/Users/randomthought/Downloads/stock_quotes_sample.csv");
        // let data_stream = file_provider::create_stream(path).unwrap();

        let mut event_stream = EventStream::new(event_channel_.clone(), data_stream, parser);
        event_stream.start().await.unwrap();
    };

    let t2 = async {
        let stream = Box::pin(event_channel.clone());
        let mut event_runner = Runner::new(event_handlers, stream);
        event_runner.run().await.unwrap()
    };

    // TODO: use scope instead https://github.com/jaboatman/tokio-scoped
    let _ = tokio::join!(t1, t2);
}
