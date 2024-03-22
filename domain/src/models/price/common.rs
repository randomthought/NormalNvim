use rust_decimal::Decimal;

pub type Symbol = String;

pub type Price = Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Second,
    Minute,
    FiveMinute,
    FifteenMinute,
    Hour,
    FourHour,
    Day,
}
