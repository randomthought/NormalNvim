use std::time::Duration;

use super::models::{Aggregates, QuoteResponse};
use domain::models::{
    price::{candle::Candle, common::Resolution, quote::Quote},
    security::{AssetType, Exchange, Security},
};
use eyre::{OptionExt, Result};
use rust_decimal::{prelude::FromPrimitive, Decimal};

pub fn to_price_history(aggregates: &Aggregates) -> Result<Candle> {
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

    let candle = Candle::builder()
        .with_security(security)
        .with_resolution(Resolution::Second)
        .with_open(Decimal::from_f64(aggregates.o).ok_or_eyre("unable to convert open to decimal")?)
        .with_high(Decimal::from_f64(aggregates.h).ok_or_eyre("unable to convert high to decimal")?)
        .with_low(Decimal::from_f64(aggregates.l).ok_or_eyre("unable to convert low to decimal")?)
        .with_close(
            Decimal::from_f64(aggregates.c).ok_or_eyre("unable to convert close to decimal")?,
        )
        .with_volume(aggregates.v)
        .with_start_time(Duration::from_millis(aggregates.s))
        .with_end_time(Duration::from_millis(aggregates.e))
        .build()?;

    Ok(candle)
}

pub fn to_quote(qoute_response: &QuoteResponse) -> Result<Quote> {
    let results = &qoute_response.results;
    let security = Security {
        asset_type: AssetType::Equity,
        exchange: Exchange::Unknown,
        ticker: results.t.to_owned(),
    };

    let quote = Quote::builder()
        .with_security(security)
        .with_bid(Decimal::from_f64(results.p).ok_or_eyre("unable to convert bid to decimal")?)
        .with_ask(Decimal::from_f64(results.p2).ok_or_eyre("unable to convert ask to decimal")?)
        .with_timestamp(Duration::from_millis(results.t2))
        .with_bid_size(results.s2)
        .with_ask_size(results.s)
        .build()?;

    Ok(quote)
}
