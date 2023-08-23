use domain::{
    engine::Engine,
    risk::{RiskEngine, RiskEngineConfig},
    strategy::{Algorithm, StrategyEngine},
};
use engine::{
    algorithms::fake_algo::FakeAlgo,
    brokers::fake_broker::FakeOrderManager,
    data_providers::{
        fake_provider::FakePriceHistoryStream,
        polygon::{models::Aggregates, polygon::PolygonClient},
    },
};
use futures_util::StreamExt;
use serde_json;

use url::Url;

#[tokio::main]
async fn main() {
    let api_key = "XXXXXX....".to_owned();
    let mut polygon_client = PolygonClient::new(api_key).await.unwrap();

    while let Some(Ok(item)) = polygon_client.next().await {
        print!("{}", item.security.ticker);
    }
}
