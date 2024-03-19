use crate::{models::security::Security, strategy::algorithm::StrategyId};

use super::common::{OrderDetails, Quantity, Side};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Market {
    pub security: Security,
    pub order_details: OrderDetails,
}

impl Market {
    // constructor
    pub fn new(
        quantity: Quantity,
        side: Side,
        security: Security,
        strategy_id: StrategyId,
    ) -> Self {
        Self {
            security,
            order_details: OrderDetails {
                quantity,
                side,
                strategy_id,
            },
        }
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}
