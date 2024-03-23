use derive_builder::Builder;
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    models::{price::common::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::common::{Quantity, Side};

#[derive(Builder, Debug, Clone, PartialEq, Eq)]
pub struct SecurityPosition {
    #[builder(setter(prefix = "with"))]
    pub security: Security,
    #[builder(setter(prefix = "with"))]
    pub side: Side,
    #[builder(setter(each = "add_holding_detail"))]
    pub holding_details: Vec<HoldingDetail>,
}

impl SecurityPosition {
    pub fn builder() -> SecurityPositionBuilder {
        SecurityPositionBuilder::default()
    }

    pub fn get_quantity(&self) -> Quantity {
        self.holding_details
            .iter()
            .fold(0, |acc, next| acc + next.quantity)
    }

    pub fn get_cost(&self) -> Decimal {
        self.holding_details
            .iter()
            .fold(Decimal::default(), |acc, next| {
                acc + (next.price * Decimal::from_u64(next.quantity).unwrap())
            })
    }
}

#[derive(Builder, Debug, Clone, PartialEq, Eq)]
pub struct HoldingDetail {
    #[builder(setter(prefix = "with"))]
    pub strategy_id: StrategyId,
    #[builder(setter(prefix = "with"))]
    pub quantity: Quantity,
    #[builder(setter(prefix = "with"))]
    pub price: Price,
}
