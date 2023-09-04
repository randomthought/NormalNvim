use rust_decimal::Decimal;

use super::security::Security;

pub type Symbol = String;
pub type Price = Decimal;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Quote {
    pub security: Security,
    pub bid: Price,
    pub bid_size: u64,
    pub ask: Price,
    pub ask_size: u64,
    pub timestamp: u64,
}

impl Quote {
    pub fn new(
        security: Security,
        bid: Price,
        ask: Price,
        bid_size: u64,
        ask_size: u64,
        timestamp: u64,
    ) -> Result<Self, String> {
        if bid > ask {
            return Err("bid price should be lower than ask price".to_owned());
        }

        Ok(Self {
            security,
            bid,
            ask,
            bid_size,
            ask_size,
            timestamp,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Resolution {
    Second,
    Minute,
    FiveMinute,
    FifteenMinute,
    Hour,
    FourHour,
    Day,
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Candle {
    pub high: Price,
    pub open: Price,
    pub low: Price,
    pub close: Price,
    // The Unix Msec timestamp for the start of the aggregate window.
    pub start_time: u64,
    pub end_time: u64,
    // The trading volume of the symbol in the given time period.
    pub volume: u64,
}

impl Candle {
    pub fn new(
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: u64,
        start_time: u64,
        end_time: u64,
    ) -> Result<Self, String> {
        if high < low {
            return Err("High cannot be less than low".to_owned());
        }

        if high < open && open < low {
            return Err("Open cannot be greater than high or less than low".to_owned());
        }

        if high < close && close < low {
            return Err("Close cannot be greater than high or less than low".to_owned());
        }

        Ok(Self {
            open,
            high,
            low,
            close,
            start_time,
            end_time,
            volume,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PriceHistory {
    pub security: Security,
    pub resolution: Resolution,
    pub history: Vec<Candle>,
}
