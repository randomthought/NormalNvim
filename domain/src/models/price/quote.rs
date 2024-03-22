use std::time::Duration;

use derive_builder::Builder;

use crate::models::security::Security;

use super::common::Price;

#[derive(Builder, Debug, Clone)]
#[non_exhaustive]
pub struct Quote {
    #[builder(setter(prefix = "with"))]
    pub security: Security,
    #[builder(setter(prefix = "with"))]
    pub bid: Price,
    #[builder(setter(prefix = "with"))]
    pub bid_size: u64,
    #[builder(setter(prefix = "with"))]
    pub ask: Price,
    #[builder(setter(prefix = "with"))]
    pub ask_size: u64,
    #[builder(setter(prefix = "with"))]
    pub timestamp: Duration,
}

impl Quote {
    pub fn builder() -> QuoteBuilder {
        QuoteBuilder::default()
    }
}

impl QuoteBuilder {
    fn validate(&self) -> Result<(), String> {
        let Some(bid) = self.bid else {
            return Err("bid must be set".into());
        };

        let Some(ask) = self.ask else {
            return Err("ask must be set".into());
        };

        if bid >= ask {
            return Err("bid price should be lower than ask price".into());
        }

        Ok(())
    }
}
