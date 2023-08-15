use domain::{
    engine::{Engine, MarketStream},
    models::{
        event::{Event, Market},
        price::{Candle, PriceHistory},
        security::{self, Security},
    },
    risk::{RiskEngine, RiskEngineConfig},
    strategy::{Algorithm, StrategyEngine},
};
use engine::{algorithms::fake_algo::FakeAlgo, brokers::fake_broker::FakeOrderManager};
use std::{io, pin::Pin, sync::Arc};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let risk_engine_config = RiskEngineConfig {
        max_portfolio_risk: 0.10,
        max_risk_per_trade: 0.05,
        max_open_trades: None,
    };

    let order_manager = FakeOrderManager {};

    let algorithms: Vec<Box<dyn Algorithm + Send + Sync>> = vec![Box::new(FakeAlgo {})];

    let t1 = tokio::spawn(async move {
        let risk_engine = RiskEngine::new(risk_engine_config, Box::new(order_manager));
        let strategy_engine = StrategyEngine::new(Box::new(risk_engine), algorithms);
        let market_stream: Pin<Box<MarketStream>> = Box::pin(MarketStream {});
        let mut algo_engine = Engine::new(strategy_engine, market_stream);
        algo_engine.runner().await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let security = Security {
            asset_type: security::AssetType::Equity,
            exchange: security::Exchange::NASDAQ,
            ticker: "AAPL".to_owned(),
        };

        let candles = vec![Candle::new(99.96, 99.98, 99.95, 99.7, 100, 888).unwrap()];

        let price_history = PriceHistory {
            security,
            resolution: domain::models::price::Resolution::Second,
            history: Box::new(candles),
        };

        let market = Market::DataEvent(price_history);
        let event = Event::Market(market);

        let mut i = 0;
        loop {
            i += 1;
            // channel2.send(event.clone()).await?;
            sleep(Duration::from_millis(500)).await;
            if i > 5 {
                break;
            }
        }

        Ok(()) as Result<(), io::Error>
    });

    t1.await.unwrap();
    t2.await.unwrap();
}
