use std::collections::BTreeSet;

use derive_builder::Builder;
use getset::Getters;

use crate::models::security::Security;

use super::{candle::Candle, common::Resolution};

#[derive(Builder, Getters)]
pub struct PriceHistory {
    #[builder(default, private)]
    #[getset(get)]
    history: BTreeSet<Candle>,
    #[builder(setter(prefix = "with"))]
    #[getset(get)]
    security: Security,
    #[builder(setter(prefix = "with"))]
    #[getset(get)]
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
