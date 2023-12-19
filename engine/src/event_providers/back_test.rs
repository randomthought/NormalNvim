use std::collections::HashMap;

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

use super::provider::Parser;

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

    fn add(&mut self, price_history: &PriceHistory) -> Result<()> {
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
        self.map.insert(ticker, q);

        Ok(())
    }
}

impl Parser for BackTestProvider {
    fn parse(
        &mut self,
        data: &str,
    ) -> anyhow::Result<Box<dyn Iterator<Item = Event> + Sync + Send>> {
        // TODO: iterator overloading mwould be better since this would be done on every price history twice

        let events = self.parser.parse(data)?;

        let mut vec: Vec<Event> = Vec::new();
        for e in events {
            if let Event::Market(Market::DataEvent(ph)) = e.clone() {
                self.add(&ph)?;
            }
            vec.push(e);
        }

        let result = Box::new(vec.into_iter());

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
