use super::{models::Aggregates, utils};
use crate::data_providers::polygon::models::{QuoteResponse, ResponseMessage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use domain::models::{
    price::{Candle, PriceHistory, Resolution},
    security::{AssetType, Exchange, Security},
};
use domain::{data::QouteProvider, models::price::Quote};
use futures_util::Stream;
use std::io;
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

static POLYGON_STOCKS_WS_API: &str = "wss://delayed.polygon.io/stocks";

pub struct PolygonClient {
    vec: Vec<PriceHistory>,
    api_key: String,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl PolygonClient {
    pub async fn new(api_key: String) -> Result<Self> {
        let (socket, _) =
            connect(Url::parse(POLYGON_STOCKS_WS_API).unwrap()).expect("Can't connect");

        let mut client = Self {
            api_key,
            socket,
            vec: Vec::new(),
        };

        client.authenticate()?;

        Ok(client)
    }

    fn authenticate(&mut self) -> Result<()> {
        let m = self.socket.read_message().expect("error connecting");
        let s = m.to_text().unwrap();
        let deserialized: Vec<ResponseMessage> = serde_json::from_str(s)
            .expect(format!("Unable to deserialize socket message: {}", s).as_str());

        println!("{:?}", deserialized);

        self.socket
            .write_message(Message::Text(
                format!(r#"{{"action":"auth","params":"{}"}}"#, self.api_key).into(),
            ))
            .expect("error when attempting to authenticate");

        // TODO: check if connection was succesful
        let m = self.socket.read_message().expect("error connecting");
        let s = m.to_text().unwrap();
        let deserialized: Vec<ResponseMessage> = serde_json::from_str(s)
            .expect(format!("Unable to deserialize socket message: {}", s).as_str());

        println!("{:?}", deserialized);

        self.socket
            .write_message(Message::Text(
                r#"{"action":"subscribe","params":"A.*"}"#.into(),
            ))
            .expect("error subscribing");

        let m = self.socket.read_message().expect("error in subscribing");
        let s = m.to_text().unwrap();
        let deserialized: Vec<ResponseMessage> = serde_json::from_str(s)
            .expect(format!("Unable to deserialize socket message: {}", s).as_str());

        println!("{:?}", deserialized);

        Ok(())
    }
}

impl Stream for PolygonClient {
    type Item = Result<PriceHistory>;

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

                if s.is_empty() {
                    return std::task::Poll::Ready(None);
                }

                let deserialized: Vec<Aggregates> = serde_json::from_str(s)
                    .expect(format!("Unable to deserialize socket message: {}", s).as_str());

                if deserialized.is_empty() {
                    return std::task::Poll::Ready(None);
                }

                for ele in deserialized {
                    let ph = utils::to_price_history(&ele);
                    self.vec.push(ph);
                }

                if let Some(item) = self.vec.pop() {
                    return std::task::Poll::Ready(Some(Ok(item)));
                }

                std::task::Poll::Ready(None)
            }
            Err(err) => {
                let err = anyhow::Error::new(err);
                println!("Error: {:?}", err);

                std::task::Poll::Ready(Some(Err(err)))
            }
        }
    }
}
