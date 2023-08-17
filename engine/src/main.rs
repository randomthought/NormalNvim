use domain::{
    engine::Engine,
    risk::{RiskEngine, RiskEngineConfig},
    strategy::{Algorithm, StrategyEngine},
};
use engine::{
    algorithms::fake_algo::FakeAlgo,
    brokers::fake_broker::FakeOrderManager,
    data_providers::{fake_provider::FakePriceHistoryStream, polygon::models::Aggregates},
};
use serde_json;

use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

static POLYGON_STOCKS_WS_API: &str = "wss://delayed.polygon.io/stocks";
#[tokio::main]
async fn main() {
    let (mut socket, response) =
        connect(Url::parse(POLYGON_STOCKS_WS_API).unwrap()).expect("Can't connect");

    let api_key = "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_owned();

    let f = format!(r#"{{"action":"auth","params":"{}"}}"#, api_key);
    socket.write_message(Message::Text(
        format!(r#"{{"action":"auth","params":"{}"}}"#, api_key).into(),
    ));

    let auth_msg = socket.read_message().expect("Error reading message");
    println!("Connection: {}", auth_msg);

    let auth_msg = socket.read_message().expect("Error reading message");
    println!("Received: {}", auth_msg);

    socket.write_message(Message::Text(
        // r#"{"action":"subscribe","params":"A.AAPL"}"#.into(),
        r#"{"action":"subscribe","params":"A.*"}"#.into(),
    ));

    let sub = socket.read_message().expect("Error reading message");
    println!("Subscribed: {}", sub);

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Recieved {:#?}", msg);
        let s = msg.to_text().unwrap();
        // print!("{}", s);
        let deserialized: Vec<Aggregates> = serde_json::from_str(s).unwrap();
        println!("Recieved {:#?}", deserialized);
    }
}
