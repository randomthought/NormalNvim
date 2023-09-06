use super::security::Security;
use anyhow::{ensure, Result};
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
    ) -> Result<Self> {
        ensure!(bid > ask, "bid price should be lower than ask price");

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
    ) -> Result<Self> {
        ensure!(high < low, "High cannot be less than low");

        ensure!(
            high < open && open < low,
            "Open cannot be greater than high or less than low"
        );

        ensure!(
            high < open && open < low,
            "Close cannot be greater than high or less than low"
        );

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
