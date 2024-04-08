use std::collections::BTreeSet;

use derive_builder::Builder;
use getset::Getters;

use crate::security::Security;

use super::{common::Resolution, price_bar::PriceBar};

#[derive(Builder, Getters)]
#[getset(get = "pub")]
#[builder(setter(prefix = "with"))]
pub struct PriceHistory {
    #[builder(default, private)]
    history: BTreeSet<PriceBar>,
    security: Security,
    resolution: Resolution,
}

impl PriceHistory {
    pub fn builder() -> PriceHistoryBuilder {
        PriceHistoryBuilder::default()
    }
}

impl PriceHistoryBuilder {
    pub fn add_candle(&mut self, value: PriceBar) -> &mut Self {
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
