use crate::data_providers::polygon::models::ResponseMessage;
use anyhow::{Context, Result};
use futures_util::Stream;
use std::{net::TcpStream, pin::Pin};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

static POLYGON_STOCKS_WS_API: &str = "wss://delayed.polygon.io/stocks";

struct PolygonStream {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl Stream for PolygonStream {
    // TODO: is they a way you can use &str instead here?
    type Item = Result<String>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        // TODO: learn to understand when to use cx
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.socket.read_message() {
            Ok(msg) => return std::task::Poll::Ready(Some(Ok(msg.to_string()))),
            Err(err) => {
                let err = anyhow::Error::new(err);
                println!("Error: {:?}", err);

                std::task::Poll::Ready(Some(Err(err)))
            }
        }
    }
}

pub fn create_stream(api_key: String) -> Result<Pin<Box<dyn Stream<Item = Result<String>>>>> {
    let (mut socket, _) =
        connect(Url::parse(POLYGON_STOCKS_WS_API).unwrap()).expect("Can't connect");

    authenticate(api_key, &mut socket)?;

    let pgs = PolygonStream { socket };

    Ok(Box::pin(pgs))
}

fn authenticate(api_key: String, socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<()> {
    let m = socket.read_message().expect("error connecting");
    let s = m.to_text().context("unable to parse connection message")?;
    let deserialized: Vec<ResponseMessage> = serde_json::from_str(s)
        .expect(format!("Unable to deserialize socket message: {}", s).as_str());

    println!("{:?}", deserialized);

    socket
        .write_message(Message::Text(
            format!(r#"{{"action":"auth","params":"{}"}}"#, api_key).into(),
        ))
        .expect("error when attempting to authenticate");

    // TODO: check if connection was succesful
    let m = socket.read_message().expect("error connecting");
    let s = m.to_text().context("unable to parse connection message")?;
    let deserialized: Vec<ResponseMessage> = serde_json::from_str(s)
        .expect(format!("Unable to deserialize socket message: {}", s).as_str());

    println!("{:?}", deserialized);

    socket
        .write_message(Message::Text(
            r#"{"action":"subscribe","params":"A.*"}"#.into(),
        ))
        .expect("error subscribing");

    let m = socket.read_message().expect("error in subscribing");
    let s = m.to_text().context("unable to parse connection message")?;
    let deserialized: Vec<ResponseMessage> = serde_json::from_str(s)
        .expect(format!("Unable to deserialize socket message: {}", s).as_str());

    println!("{:?}", deserialized);

    Ok(())
}
