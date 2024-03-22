use derive_builder::Builder;

pub type Ticker = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Exchange {
    // TODO: add list of exchanges
    NASDAQ,
    NYSE,
    AMEX,
    OTC,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Equity,
    Forex,
    Future,
    Option,
    Crypto,
}

#[derive(Builder, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Security {
    #[builder(setter(prefix = "with"))]
    pub asset_type: AssetType,
    #[builder(setter(prefix = "with"))]
    pub exchange: Exchange,
    #[builder(setter(prefix = "with"))]
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

    pub fn builder() -> SecurityBuilder {
        SecurityBuilder::default()
    }
}
