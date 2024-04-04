use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use data_providers::parser::{Parser, ParserError};
use domain::event::model::DataEvent;
use domain::models::price::candle::PriceBar;
use domain::models::price::quote::Quote;
use domain::{data::QouteProvider, models::security::Security};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tokio::sync::RwLock;

pub struct BackTester {
    spread: Decimal,
    map: Arc<RwLock<HashMap<String, Quote>>>,
    parser: Box<dyn Parser + Sync + Send>,
}

impl BackTester {
    pub fn new(spread: f64, parser: Box<dyn Parser + Sync + Send>) -> Self {
        let hash_map = HashMap::new();
        let map = Arc::new(RwLock::new(hash_map));
        Self {
            // TODO: should we consider error handling if we cannot parse floating?
            spread: Decimal::from_f64(spread).unwrap(),
            map,
            parser,
        }
    }

    async fn add(&self, candle: &PriceBar) -> Result<()> {
        let spread_half = (candle.close * self.spread) / Decimal::from(2);
        let bid = candle.close - spread_half;
        let ask = candle.close + spread_half;
        let ticker = candle.security.ticker.clone();
        let security = candle.security.clone();

        let q = Quote::builder()
            .with_security(security)
            .with_bid(bid)
            .with_ask(ask)
            .with_ask_size(0)
            .with_bid_size(0)
            .with_timestamp(candle.end_time)
            .build()
            .map_err(|e| eyre::Report::msg(e))?;

        let mut map = self.map.write().await;
        map.insert(ticker, q);

        Ok(())
    }
}

#[async_trait]
impl Parser for BackTester {
    async fn parse(&self, data: &str) -> Result<Option<DataEvent>, ParserError> {
        let event = self.parser.parse(data).await?;
        let Some(event) = event else { return Ok(None) };
        let DataEvent::Candle(ph) = event.clone();
        self.add(&ph)
            .await
            .map_err(|e| ParserError::OtherError(e.into()))?;

        Ok(Some(event))
    }
}

#[async_trait]
impl QouteProvider for BackTester {
    async fn get_quote(&self, security: &Security) -> Result<Quote, domain::error::Error> {
        let map = self.map.read().await;
        let quote = map.get(&security.ticker).ok_or_else(|| {
            domain::error::Error::Message(format!(
                "security='{}' not found in map",
                security.ticker
            ))
        })?;

        Ok(quote.clone())
    }
}
