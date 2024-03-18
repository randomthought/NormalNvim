use crate::strategy::algorithm::StrategyId;

use super::{common::OrderId, new_order::NewOrder, order_result::OrderResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    NewOrder(NewOrder),
    OrderResult(OrderResult),
}

impl Order {
    pub fn startegy_id(&self) -> StrategyId {
        match self {
            Order::NewOrder(o) => o.startegy_id(),
            Order::OrderResult(o) => o.startegy_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub order: NewOrder,
}

impl PendingOrder {
    pub fn startegy_id(&self) -> StrategyId {
        self.order.startegy_id()
    }
}
