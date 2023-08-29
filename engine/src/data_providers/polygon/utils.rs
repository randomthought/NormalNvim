use domain::models::{
    price::{Candle, PriceHistory, Quote, Resolution},
    security::{AssetType, Exchange, Security},
};

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
        aggregates.o,
        aggregates.h,
        aggregates.l,
        aggregates.c,
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
        security, results.p, results.p2, results.s2, results.s, results.t2,
    )
    .unwrap()
}
