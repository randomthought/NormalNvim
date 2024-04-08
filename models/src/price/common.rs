use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type Symbol = String;

pub type Price = Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    Second,
    Minute,
    FiveMinute,
    FifteenMinute,
    Hour,
    FourHour,
    Day,
}
