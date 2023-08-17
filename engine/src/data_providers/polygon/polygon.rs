use std::net::TcpStream;

use domain::models::price::PriceHistory;
use futures_util::Stream;

use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

static POLYGON_STOCKS_WS_API: &str = "wss://delayed.polygon.io/stocks";

use async_stream::stream;

pub struct PolygonClient {
    api_key: String,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    authenticated: bool,
}

impl PolygonClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            socket: None,
            authenticated: false,
        }
    }
}

impl Stream for PolygonClient {
    type Item = PriceHistory;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!();
    }
}

fn zero_to_three() -> impl Stream<Item = u32> {
    stream! {
        for i in 0..3 {
            yield i;
        }
    }
}
