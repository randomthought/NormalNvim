use std::collections::BTreeSet;

use derive_builder::Builder;
use derive_getters::Getters;

use crate::models::security::Security;

use super::{candle::Candle, common::Resolution};

#[derive(Builder, Getters)]
#[builder(setter(prefix = "with"))]
pub struct PriceHistory {
    #[builder(default, private)]
    history: BTreeSet<Candle>,
    security: Security,
    resolution: Resolution,
}

impl PriceHistory {
    pub fn builder() -> PriceHistoryBuilder {
        PriceHistoryBuilder::default()
    }
}

impl PriceHistoryBuilder {
    pub fn add_candle(&mut self, value: Candle) -> &mut Self {
        self.security = Some(value.clone().security);
        self.resolution = Some(value.clone().resolution);
        if let Some(history) = self.history.as_mut() {
            history.insert(value.clone());
            return self;
        }

        let mut history = BTreeSet::default();
        history.insert(value);
        self.history = Some(history);

        todo!()
    }
}
