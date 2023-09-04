use domain::models::{
    price::{Candle, PriceHistory, Quote, Resolution},
    security::{AssetType, Exchange, Security},
};
use rust_decimal::{prelude::FromPrimitive, Decimal};

use super::models::{Aggregates, QuoteResponse};

pub fn to_price_history(aggregates: &Aggregates) -> PriceHistory {
    let exchange = if aggregates.otc {
        Exchange::OTC
    } else {
        Exchange::Unkown
    };

    let security = Security {
        asset_type: AssetType::Equity,
        exchange,
        ticker: aggregates.sym.to_owned(),
    };

    let candle = Candle::new(
        Decimal::from_f64(aggregates.o).unwrap(),
        Decimal::from_f64(aggregates.h).unwrap(),
        Decimal::from_f64(aggregates.l).unwrap(),
        Decimal::from_f64(aggregates.c).unwrap(),
        aggregates.v,
        aggregates.s,
        aggregates.e,
    )
    .unwrap();

    let history = vec![candle];

    PriceHistory {
        security,
        history,
        resolution: Resolution::Second,
    }
}

pub fn to_quote(qoute_response: &QuoteResponse) -> Quote {
    let results = &qoute_response.results;
    let security = Security {
        asset_type: AssetType::Equity,
        exchange: Exchange::Unkown,
        ticker: results.t.to_owned(),
    };

    Quote::new(
        security,
        Decimal::from_f64(results.p).unwrap(),
        Decimal::from_f64(results.p2).unwrap(),
        results.s2,
        results.s,
        results.t2,
    )
    .unwrap()
}
