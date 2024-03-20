use derive_builder::Builder;

use crate::{
    models::{price::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::common::{OrderDetails, Quantity, Side, TimeInForce};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limit {
    pub price: Price,
    pub security: Security,
    pub times_in_force: TimeInForce,
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

#[derive(Builder)]
#[builder(public, name = "LimitBuilder", build_fn(private, name = "build_seed",))]
struct LimitSeed {
    #[builder(setter(prefix = "with"))]
    pub price: Price,
    #[builder(setter(prefix = "with"))]
    pub security: Security,
    #[builder(setter(prefix = "with"))]
    pub times_in_force: TimeInForce,
    #[builder(setter(prefix = "with"))]
    pub strategy_id: StrategyId,
    #[builder(setter(prefix = "with"))]
    pub quantity: Quantity,
    #[builder(setter(prefix = "with"))]
    pub side: Side,
}

impl LimitSeed {
    fn build(&self) -> Result<Limit, LimitBuilderError> {
        let order_details = OrderDetails::builder()
            .with_strategy_id(self.strategy_id)
            .with_quantity(self.quantity)
            .with_side(self.side)
            .build()
            .map_err(|e| e.to_string())?;

        let limit = Limit {
            order_details,
            price: self.price.to_owned(),
            security: self.security.to_owned(),
            times_in_force: self.times_in_force,
        };

        Ok(limit)
    }
}

impl LimitBuilder {
    pub fn build(&self) -> Result<Limit, LimitBuilderError> {
        let seed = self.build_seed()?;
        seed.build()
    }
}
