use core::time;
use std::thread::sleep;

use domain::models::{
    price::{Candle, PriceHistory},
    security::{self, Security},
};
use futures_util::stream::Stream;
use rust_decimal::{prelude::FromPrimitive, Decimal};

pub struct FakePriceHistoryStream {
    pub max: i32,
}

impl Stream for FakePriceHistoryStream {
    type Item = PriceHistory;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.max < 0 {
            return std::task::Poll::Ready(None);
        }

        self.max -= 1;

        let security = Security {
            asset_type: security::AssetType::Equity,
            exchange: security::Exchange::NASDAQ,
            ticker: "AAPL".to_owned(),
        };

        let candles = vec![Candle::new(
            Decimal::from_f64(99.96).unwrap(),
            Decimal::from_f64(99.98).unwrap(),
            Decimal::from_f64(99.95).unwrap(),
            Decimal::from_f64(99.7).unwrap(),
            100,
            888,
            0,
        )
        .unwrap()];

        let price_history = PriceHistory {
            security,
            resolution: domain::models::price::Resolution::Second,
            history: candles,
        };

        sleep(time::Duration::from_millis(500));

        return std::task::Poll::Ready(Some(price_history));
    }
}
