pub type Symbol = String;
pub type Price = f32;

#[derive(Debug)]
pub struct Candle {
    high: Price,
    open: Price,
    low: Price,
    close: Price,
    // The Unix Msec timestamp for the start of the aggregate window.
    time: i32,
    // The trading volume of the symbol in the given time period.
    volume: i32,
}

impl Candle {
    pub fn new(
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: i32,
        time: i32,
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

pub struct EquityHistory<'a> {
    symbol: Symbol,
    history: Vec<&'a Candle>,
}
