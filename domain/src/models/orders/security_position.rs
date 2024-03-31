use derive_builder::Builder;
use getset::Getters;
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

#[derive(Builder, Getters, Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub")]
#[builder(setter(prefix = "with"))]
pub struct HoldingDetail {
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub price: Price,
}

impl HoldingDetail {
    pub fn builder() -> HoldingDetailBuilder {
        HoldingDetailBuilder::default()
    }
}
