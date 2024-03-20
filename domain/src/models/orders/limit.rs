use derive_builder::Builder;

use crate::{
    models::{price::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::common::{OrderDetails, Side, TimeInForce};

#[derive(Builder, Debug, Clone, PartialEq, Eq)]
pub struct Limit {
    #[builder(setter(prefix = "with"))]
    pub price: Price,
    #[builder(setter(prefix = "with"))]
    pub security: Security,
    #[builder(setter(prefix = "with"))]
    pub times_in_force: TimeInForce,
    #[builder(setter(prefix = "with"))]
    pub order_details: OrderDetails,
}

impl Limit {
    pub fn builder() -> LimitBuilder {
        LimitBuilder::default()
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}
