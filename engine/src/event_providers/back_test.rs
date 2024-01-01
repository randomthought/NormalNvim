use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use domain::event::model::Event;
use domain::event::model::Market;
use domain::{
    data::QouteProvider,
    models::{
        price::{PriceHistory, Quote},
        security::Security,
    },
};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tokio::sync::RwLock;

use super::provider::Parser;

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
        let c = price_history
            .history
            .last()
            .context("no price history in 'security={ph.security.ticker}")?;

        let spread_half = (c.close * self.spread) / Decimal::from(2);
        let bid = c.close - spread_half;
        let ask = c.close + spread_half;
        let ticker = price_history.security.ticker.clone();
        let security = price_history.security.clone();

        let q = Quote::new(security, bid, ask, 0, 0, c.end_time)?;
        let mut map = self.map.write().await;
        map.insert(ticker, q);

        Ok(())
    }
}

#[async_trait]
impl Parser for BackTester {
    async fn parse(&self, data: &str) -> anyhow::Result<Vec<Event>> {
        // TODO: iterator overloading mwould be better since this would be done on every price history twice

        let events = self.parser.parse(data).await?;

        let mut vec: Vec<Event> = Vec::new();
        for e in events {
            if let Event::Market(Market::DataEvent(ph)) = e.clone() {
                self.add(&ph).await?;
            }
            vec.push(e);
        }

        Ok(vec)
    }
}

#[async_trait]
impl QouteProvider for BackTester {
    async fn get_quote(&self, security: &Security) -> Result<Quote> {
        let map = self.map.read().await;
        let quote = map
            .get(&security.ticker)
            .context("security '{security.ticker}' not found in map")?
            .clone();

        Ok(quote)
    }
}
