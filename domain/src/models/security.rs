pub type Ticker = String;

#[derive(Debug, Clone, Copy)]
pub enum Exchange {
    // TODO: add list of exchanges
    NASDAQ,
    NYSE,
    AMEX,
    OTC,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum AssetType {
    Equity,
    Forex,
    Future,
    Option,
    Crypto,
}

#[derive(Debug, Clone)]
pub struct Security {
    pub asset_type: AssetType,
    pub exchange: Exchange,
    pub ticker: Ticker,
}

impl Security {
    pub fn new(asset_type: AssetType, exchange: Exchange, ticker: String) -> Self {
        Security {
            asset_type,
            exchange,
            ticker,
        }
    }
}
