use std::time::Duration;

use super::models::{Aggregates, QuoteResponse};
use anyhow::{Context, Ok, Result};
use domain::models::{
    price::{Candle, PriceHistory, Quote, Resolution},
    security::{AssetType, Exchange, Security},
};
use rust_decimal::{prelude::FromPrimitive, Decimal};

pub fn to_price_history(aggregates: &Aggregates) -> Result<PriceHistory> {
    let exchange = if aggregates.otc {
        Exchange::OTC
    } else {
        Exchange::Unknown
    };

    let security = Security {
        asset_type: AssetType::Equity,
        exchange,
        ticker: aggregates.sym.to_owned(),
    };

    let candle = Candle::new(
        Decimal::from_f64(aggregates.o).context("unable to convert open to decimal")?,
        Decimal::from_f64(aggregates.h).context("unable to convert high to decimal")?,
        Decimal::from_f64(aggregates.l).context("unable to convert low to decimal")?,
        Decimal::from_f64(aggregates.c).context("unable to convert close to decimal")?,
        aggregates.v,
        Duration::from_millis(aggregates.s),
        Duration::from_millis(aggregates.e),
    )?;

    let history = vec![candle];

    let price_history = PriceHistory {
        security,
        history,
        resolution: Resolution::Second,
    };

    Ok(price_history)
}

pub fn to_quote(qoute_response: &QuoteResponse) -> Result<Quote> {
    let results = &qoute_response.results;
    let security = Security {
        asset_type: AssetType::Equity,
        exchange: Exchange::Unknown,
        ticker: results.t.to_owned(),
    };

    let quote = Quote::new(
        security,
        Decimal::from_f64(results.p).context("unable to convert bid to decimal")?,
        Decimal::from_f64(results.p2).context("unable to convert ask to decimal")?,
        results.s2,
        results.s,
        Duration::from_millis(results.t2),
    )?;

    Ok(quote)
}
