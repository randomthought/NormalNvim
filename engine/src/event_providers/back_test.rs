use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use domain::event::model::DataEvent;
use domain::{
    data::QouteProvider,
    models::{
        price::{PriceHistory, Quote},
        security::Security,
    },
};
use eyre::ContextCompat;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tokio::sync::RwLock;

use super::provider::Parser;
use super::provider::ParserError;

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

    async fn add(&self, price_history: &PriceHistory) -> Result<()> {
        let c = price_history.history.last().wrap_err(format!(
            "no price history in 'security={}",
            price_history.security.ticker
        ))?;

        let spread_half = (c.close * self.spread) / Decimal::from(2);
        let bid = c.close - spread_half;
        let ask = c.close + spread_half;
        let ticker = price_history.security.ticker.clone();
        let security = price_history.security.clone();

        let q =
            Quote::new(security, bid, ask, 0, 0, c.end_time).map_err(|e| eyre::Report::msg(e))?;

        let mut map = self.map.write().await;
        map.insert(ticker, q);

        Ok(())
    }
}

#[async_trait]
impl Parser for BackTester {
    async fn parse(&self, data: &str) -> Result<DataEvent, ParserError> {
        let event = self.parser.parse(data).await?;
        let DataEvent::PriceEvent(ph) = event.clone();
        self.add(&ph)
            .await
            .map_err(|e| ParserError::OtherError(e.into()))?;

        Ok(event)
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
