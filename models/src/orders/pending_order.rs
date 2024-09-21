use derive_builder::Builder;
use getset::Getters;

use crate::strategy::common::StrategyId;

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

#[derive(Debug, Getters, Builder, Clone, PartialEq, Eq)]
#[getset(get = "pub")]
#[builder(public, setter(prefix = "with"))]
// TODO: pending order should have a time stamp
pub struct PendingOrder {
    order_id: OrderId,
    order: NewOrder,
}

impl PendingOrder {
    pub fn builder() -> PendingOrderBuilder {
        PendingOrderBuilder::default()
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order.startegy_id()
    }
}
