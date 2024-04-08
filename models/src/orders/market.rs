use derive_builder::Builder;

use crate::{security::Security, strategy::common::StrategyId};

use super::common::{OrderDetails, Quantity, Side};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Market {
    pub security: Security,
    pub order_details: OrderDetails,
}

impl Market {
    pub fn builder() -> MarketBuilder {
        MarketBuilder::default()
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order_details.strategy_id()
    }
}

#[derive(Builder)]
#[builder(
    public,
    name = "MarketBuilder",
    build_fn(private, name = "build_seed",)
)]
#[builder(setter(prefix = "with"))]
struct MarketSeed {
    pub security: Security,
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub side: Side,
}

impl MarketSeed {
    fn build(&self) -> Result<Market, MarketBuilderError> {
        let order_details = OrderDetails::builder()
            .with_strategy_id(self.strategy_id)
            .with_quantity(self.quantity)
            .with_side(self.side)
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Market {
            order_details,
            security: self.security.to_owned(),
        })
    }
}

impl MarketBuilder {
    pub fn build(&self) -> Result<Market, MarketBuilderError> {
        let seed = self.build_seed()?;
        seed.build()
    }
}
