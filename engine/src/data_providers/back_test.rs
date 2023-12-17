use core::f64;
use std::collections::HashMap;

use anyhow::Context;
use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use domain::{
    data::QouteProvider,
    engine::Parser,
    models::{
        price::{PriceHistory, Quote},
        security::Security,
    },
};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

struct BackTestProvider {
    spread: Decimal,
    map: HashMap<String, Quote>,
    parser: Box<dyn Parser + Sync + Send>,
}

impl BackTestProvider {
    pub fn new(spread: f64, parser: Box<dyn Parser + Sync + Send>) -> Self {
        Self {
            // TODO: should we consider error handling if we cannot parse floating?
            spread: Decimal::from_f64(spread).unwrap(),
            map: HashMap::new(),
            parser,
        }
    }

    fn add(&self, price_history: &PriceHistory) -> Result<()> {
        let c = price_history
            .history
            .last()
            .context("no price history in 'security={ph.security.ticker}")?;

        let spread = (c.close * self.spread) / Decimal::from(2);
        let bid = c.close - spread;
        let ask = c.close + spread;
        let q = Quote::new(price_history.security, bid, ask, 0, 0, c.end_time)?;

        self.map.insert(price_history.security.ticker, q);

        Ok(())
    }
}

impl Parser for BackTestProvider {
    fn parse(&self, data: &str) -> anyhow::Result<Box<dyn Iterator<Item = PriceHistory>>> {
        // TODO: iterator overloading mwould be better since this would be done on every price history twice
        let price_histories: Vec<_> = self.parser.parse(data)?.collect();

        for ph in price_histories {
            self.add(&ph)?
        }

        let result = Box::new(price_histories.into_iter());

        Ok(result)
    }
}

#[async_trait]
impl QouteProvider for BackTestProvider {
    async fn get_quote(&self, security: &Security) -> Result<Quote> {
        let quote = self
            .map
            .get(&security.ticker)
            .context("security '{security.ticker}' not found in map")?
            .clone();

        Ok(quote)
    }
}

