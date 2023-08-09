use super::security::Security;

pub type Symbol = String;
pub type Price = f32;

#[derive(Debug)]
#[non_exhaustive]
pub struct Quote {
    pub bid: f32,
    pub ask: f32,
    pub volume: u32,
}

impl Quote {
    pub fn new(bid: f32, ask: f32, volume: u32) -> Result<Self, String> {
        if bid > ask {
            return Err("bid price should be lower than ask price".to_owned());
        }

        Ok(Self { bid, ask, volume })
    }
}

#[derive(Debug)]
pub enum Resolution {
    Second,
    Minute,
    FiveMinute,
    FifteenMinute,
    Hour,
    FourHour,
    Day,
}

#[derive(Debug)]
pub struct Candle {
    high: Price,
    open: Price,
    low: Price,
    close: Price,
    // The Unix Msec timestamp for the start of the aggregate window.
    time: u32,
    // The trading volume of the symbol in the given time period.
    volume: u32,
}

impl Candle {
    pub fn new(
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: u32,
        time: u32,
    ) -> Result<Self, String> {
        if high < low {
            return Err("High cannot be less than low".to_owned());
        }

        if open > high && open < low {
            return Err("Open cannot be greater than high or less than low".to_owned());
        }

        if open > close && close < low {
            return Err("Close cannot be greater than high or less than low".to_owned());
        }

        Ok(Self {
            open,
            high,
            low,
            close,
            time,
            volume,
        })
    }
}

#[derive(Debug)]
pub struct PriceHistory {
    pub security: Security,
    pub resolution: Resolution,
    pub history: Box<[Candle]>,
}

// impl<'a> PriceHistory<'a> {
//     pub fn new(security: &'a Security, resolution: Resolution, history: &'a [&'a Candle]) -> Self {
//         Ok(Self {
//             security,
//             resolution,
//             // TODO: ensure you sort
//             history,
//         })
//     }
// }
