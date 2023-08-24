use std::io;
use std::net::TcpStream;

use domain::models::{
    price::{Candle, PriceHistory, Resolution},
    security::{AssetType, Exchange, Security},
};
use futures_util::Stream;

use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

static POLYGON_STOCKS_WS_API: &str = "wss://delayed.polygon.io/stocks";

use super::models::Aggregates;

pub struct PolygonClient {
    vec: Vec<PriceHistory>,
    api_key: String,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    authenticated: bool,
}

impl PolygonClient {
    pub async fn new(api_key: String) -> Result<Self, io::Error> {
        let (socket, _) =
            connect(Url::parse(POLYGON_STOCKS_WS_API).unwrap()).expect("Can't connect");

        let mut client = Self {
            api_key,
            socket,
            vec: Vec::new(),
            authenticated: false,
        };

        client.authenticate()?;

        Ok(client)
    }

    fn authenticate(&mut self) -> Result<(), io::Error> {
        let m = self.socket.read_message().expect("Error connecting");
        println!("{}", m);

        self.socket.write_message(Message::Text(
            format!(r#"{{"action":"auth","params":"{}"}}"#, self.api_key).into(),
        ));

        // TODO: check if connection was succesful
        let m = self.socket.read_message().expect("Error connecting");
        println!("{}", m);

        self.socket.write_message(Message::Text(
            r#"{"action":"subscribe","params":"A.*"}"#.into(),
        ));

        let m = self.socket.read_message().expect("Error in subscribing");
        println!("{}", m);

        self.authenticated = true;

        Ok(())
    }
}

impl Stream for PolygonClient {
    type Item = Result<PriceHistory, io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if let Some(item) = self.vec.pop() {
            return std::task::Poll::Ready(Some(Ok(item)));
        }

        match self.socket.read_message() {
            Ok(msg) => {
                let s = msg.to_text().unwrap();
                let deserialized: Vec<Aggregates> = serde_json::from_str(s).unwrap();
                if deserialized.is_empty() {
                    return std::task::Poll::Ready(None);
                }

                for ele in deserialized {
                    let ph = convert(ele);
                    self.vec.push(ph);
                }

                if let Some(item) = self.vec.pop() {
                    println!("{:?}", item);
                    return std::task::Poll::Ready(Some(Ok(item)));
                }

                std::task::Poll::Ready(None)
            }
            Err(err) => {
                let err = io::Error::new(io::ErrorKind::Other, err.to_string());
                std::task::Poll::Ready(Some(Err(err)))
            }
        }
    }
}

fn convert(aggregates: Aggregates) -> PriceHistory {
    let security = Security {
        asset_type: AssetType::Equity,
        exchange: Exchange::Unkown,
        ticker: aggregates.sym,
    };

    let candle = Candle::new(
        aggregates.o,
        aggregates.h,
        aggregates.l,
        aggregates.c,
        convert_volume(aggregates.v),
        convert_volume(0), // TODO: get the actual time from the API
    )
    .unwrap();

    let history = vec![candle];

    PriceHistory {
        security,
        history,
        resolution: Resolution::Second,
    }
}

fn convert_volume(x: i64) -> u64 {
    (x as u64) ^ (1 << 63)
}
