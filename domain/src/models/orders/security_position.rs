use crate::{
    models::{price::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::common::{Quantity, Side};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPosition {
    pub security: Security,
    pub side: Side,
    pub holding_details: Vec<HoldingDetail>,
}

impl SecurityPosition {
    pub fn get_quantity(&self) -> Quantity {
        self.holding_details
            .iter()
            .fold(0, |acc, next| acc + next.quantity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoldingDetail {
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub price: Price,
}
