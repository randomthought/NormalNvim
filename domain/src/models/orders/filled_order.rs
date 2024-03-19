use std::time::Duration;

use crate::{
    models::{price::Price, security::Security},
    strategy::algorithm::StrategyId,
};

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
    pub fn new(
        security: Security,
        order_id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
        date_time: Duration,
        strategy_id: StrategyId,
    ) -> Self {
        let order_details = OrderDetails {
            quantity,
            side,
            strategy_id,
        };
        Self {
            security,
            order_id,
            price,
            date_time,
            order_details,
        }
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}
