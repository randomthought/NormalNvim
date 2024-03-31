use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    models::{price::common::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::common::{Quantity, Side};

#[derive(Builder, Getters, CopyGetters, Debug, Clone, PartialEq, Eq)]
pub struct SecurityPosition {
    #[getset(get = "pub")]
    #[builder(setter(prefix = "with"))]
    pub security: Security,
    #[getset(get_copy = "pub")]
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

    pub fn get_wieghted_average_price(&self) -> Decimal {
        let quantity = Decimal::from_u64(self.get_quantity()).unwrap();
        self.get_cost() / quantity
    }

    pub fn get_cost(&self) -> Decimal {
        self.holding_details
            .iter()
            .fold(Decimal::default(), |acc, next| {
                acc + (next.price * Decimal::from_u64(next.quantity).unwrap())
            })
    }
}

#[derive(Builder, Getters, CopyGetters, Debug, Clone, PartialEq, Eq)]
#[builder(setter(prefix = "with"))]
pub struct HoldingDetail {
    #[getset(get = "pub")]
    pub strategy_id: StrategyId,
    #[getset(get_copy = "pub")]
    pub quantity: Quantity,
    #[getset(get = "pub")]
    pub price: Price,
}

impl HoldingDetail {
    pub fn builder() -> HoldingDetailBuilder {
        HoldingDetailBuilder::default()
    }
}
