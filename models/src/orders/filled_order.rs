use std::time::Duration;

use derive_builder::Builder;

use crate::{price::common::Price, security::Security, strategy::common::StrategyId};

use super::common::{OrderDetails, OrderId, Quantity, Side};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FilledOrder {
    pub security: Security,
    pub order_id: OrderId,
    pub price: Price,
    pub date_time: Duration,
    pub order_details: OrderDetails,
}

impl FilledOrder {
    pub fn builder() -> FilledOrderBuilder {
        FilledOrderBuilder::default()
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order_details.strategy_id()
    }
}

#[derive(Builder)]
#[builder(
    public,
    name = "FilledOrderBuilder",
    build_fn(private, name = "build_seed",)
)]
#[builder(setter(prefix = "with"))]
struct FilledOrderSeed {
    pub security: Security,
    pub order_id: OrderId,
    pub price: Price,
    pub date_time: Duration,
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub side: Side,
}

impl FilledOrderSeed {
    fn build(&self) -> Result<FilledOrder, FilledOrderBuilderError> {
        let order_details = OrderDetails::builder()
            .with_side(self.side)
            .with_quantity(self.quantity)
            .with_strategy_id(self.strategy_id)
            .build()
            .map_err(|e| e.to_string())?;

        Ok(FilledOrder {
            order_details,
            security: self.security.to_owned(),
            date_time: self.date_time,
            price: self.price.to_owned(),
            order_id: self.order_id.to_owned(),
        })
    }
}

impl FilledOrderBuilder {
    pub fn build(&self) -> Result<FilledOrder, FilledOrderBuilderError> {
        let seed = self.build_seed()?;
        seed.build()
    }
}
