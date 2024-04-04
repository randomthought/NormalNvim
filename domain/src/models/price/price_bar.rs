use std::{cmp::Ordering, time::Duration};

use derive_builder::Builder;
use serde::{Deserialize, Deserializer, Serialize};

use crate::models::{
    security::Security,
    utils::{deserialize_duration_from_unix_timestamp, serialize_duration_in_millis},
};

use super::common::{Price, Resolution};

#[derive(Builder, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[builder(setter(prefix = "with"))]
#[non_exhaustive]
pub struct PriceBar {
    pub security: Security,
    pub resolution: Resolution,
    pub high: Price,
    pub open: Price,
    pub low: Price,
    pub close: Price,
    #[serde(deserialize_with = "deserialize_duration_from_unix_timestamp")]
    #[serde(serialize_with = "serialize_duration_in_millis")]
    pub start_time: Duration,
    #[serde(deserialize_with = "deserialize_duration_from_unix_timestamp")]
    #[serde(serialize_with = "serialize_duration_in_millis")]
    pub end_time: Duration,
    // The trading volume of the symbol in the given time period.
    pub volume: u64,
}

impl PriceBar {
    pub fn builder() -> PriceBarBuilder {
        PriceBarBuilder::default()
    }
}

impl PartialOrd for PriceBar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other)) // Delegate to the implementation of `cmp`.
    }
}

impl Ord for PriceBar {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_time.cmp(&other.start_time)
    }
}

impl PriceBarBuilder {
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
