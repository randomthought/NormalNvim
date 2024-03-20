use crate::{
    models::{price::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::common::{OrderDetails, Side, TimeInForce};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limit {
    pub price: Price,
    pub security: Security,
    pub times_in_force: TimeInForce,
    pub order_details: OrderDetails,
}

impl Limit {
    pub fn new(
        quantity: u64,
        price: Price,
        side: Side,
        security: Security,
        times_in_force: TimeInForce,
        strategy_id: StrategyId,
    ) -> Self {
        Self {
            price,
            security,
            times_in_force,
            order_details: OrderDetails {
                quantity,
                side,
                strategy_id,
            },
        }
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}
