type Ticker = String;

#[derive(Debug)]
pub enum Exchange {
    // TODO: add list of exchanges
    NASDAQ,
    NYSE,
}

#[derive(Debug)]
pub enum AssetType {
    Equity,
    Forex,
    Future,
    Option,
    Crypto,
}

#[derive(Debug)]
pub struct Security {
    asset_type: AssetType,
    exchange: Exchange,
    ticker: Ticker,
}

impl Security {
    pub fn new(ticker: Ticker, exchange: Exchange, asset_type: AssetType) -> Self {
        Security {
            asset_type,
            exchange,
            ticker,
        }
    }
}
