use std::time::Duration;

use derive_builder::Builder;

use crate::models::security::Security;

use super::common::{Price, Resolution};

#[derive(Builder, Debug, Clone)]
#[non_exhaustive]
pub struct Candle {
    #[builder(setter(prefix = "with"))]
    pub security: Security,
    #[builder(setter(prefix = "with"))]
    pub resolution: Resolution,
    #[builder(setter(prefix = "with"))]
    pub high: Price,
    #[builder(setter(prefix = "with"))]
    pub open: Price,
    #[builder(setter(prefix = "with"))]
    pub low: Price,
    #[builder(setter(prefix = "with"))]
    pub close: Price,
    #[builder(setter(prefix = "with"))]
    pub start_time: Duration,
    #[builder(setter(prefix = "with"))]
    pub end_time: Duration,
    // The trading volume of the symbol in the given time period.
    #[builder(setter(prefix = "with"))]
    pub volume: u64,
}

impl Candle {
    pub fn builder() -> CandleBuilder {
        CandleBuilder::default()
    }
}

impl CandleBuilder {
    fn validate(&self) -> Result<(), String> {
        let Some(high) = self.high else {
            return Err("high must be set".into());
        };
        let Some(low) = self.low else {
            return Err("low must be set".into());
        };
        let Some(close) = self.close else {
            return Err("close must be set".into());
        };
        let Some(open) = self.open else {
            return Err("open must be set".into());
        };

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

        Ok(())
    }
}
