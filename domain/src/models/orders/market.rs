use derive_builder::Builder;

use crate::{models::security::Security, strategy::algorithm::StrategyId};

use super::common::{OrderDetails, Quantity, Side};

#[derive(Builder, Debug, Clone, PartialEq, Eq)]
pub struct Market {
    #[builder(setter(prefix = "with"))]
    pub security: Security,
    #[builder(setter(prefix = "with"))]
    pub order_details: OrderDetails,
}

impl Market {
    pub fn builder() -> MarketBuilder {
        MarketBuilder::default()
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}
