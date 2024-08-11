use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use models::price::price_bar::PriceBar;
use models::price::quote::Quote;
use models::security::Security;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tokio::sync::RwLock;
use traits::data::QouteProvider;

pub struct InMemoryQouteProvider {
    spread: Decimal,
    map: Arc<RwLock<HashMap<String, Quote>>>,
}

impl InMemoryQouteProvider {
    pub fn new(spread: f64) -> Self {
        let hash_map = HashMap::new();
        let map = Arc::new(RwLock::new(hash_map));
        Self {
            // TODO: should we consider error handling if we cannot parse floating?
            spread: Decimal::from_f64(spread).unwrap(),
            map,
        }
    }

    pub async fn add(&self, candle: &PriceBar) -> Result<()> {
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
impl QouteProvider for InMemoryQouteProvider {
    async fn get_quote(&self, security: &Security) -> Result<Quote, models::error::Error> {
        let map = self.map.read().await;
        let quote = map.get(&security.ticker).ok_or_else(|| {
            models::error::Error::Message(format!(
                "security='{}' not found in map",
                security.ticker
            ))
        })?;

        Ok(quote.clone())
    }
}
