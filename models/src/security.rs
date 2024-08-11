use derive_builder::Builder;
use serde::{Deserialize, Serialize};

pub type Ticker = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Exchange {
    // TODO: add list of exchanges
    NASDAQ,
    NYSE,
    AMEX,
    OTC,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Equity,
    Forex,
    Future,
    Option,
    Crypto,
}

#[derive(Builder, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[builder(setter(prefix = "with"))]
pub struct Security {
    pub asset_type: AssetType,
    pub exchange: Exchange,
    pub ticker: Ticker,
}

impl Security {
    pub fn builder() -> SecurityBuilder {
        SecurityBuilder::default()
    }
}
