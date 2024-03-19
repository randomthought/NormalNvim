use std::time::Duration;

use super::security::Security;
use rust_decimal::Decimal;

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
    pub timestamp: Duration,
}

impl Quote {
    pub fn new(
        security: Security,
        bid: Price,
        ask: Price,
        bid_size: u64,
        ask_size: u64,
        timestamp: Duration,
    ) -> Result<Self, String> {
        if bid >= ask {
            return Err("bid price should be lower than ask price".into());
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
    pub start_time: Duration,
    pub end_time: Duration,
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
        start_time: Duration,
        end_time: Duration,
    ) -> Result<Self, String> {
        if high < low {
            return Err("high cannot be less than low".into());
        }

        if high < open {
            return Err("open cannot be greater than high".into());
        }

        if open < low {
            return Err("open cannot be less than low".into());
        }

        if close < low {
            return Err("close cannot be less than low".into());
        }

        if high < close {
            return Err("close cannot be greater than high".into());
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
