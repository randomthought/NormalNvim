use derive_builder::Builder;
use getset::Getters;
use strum_macros::{AsRefStr, VariantNames};

use crate::strategy::algorithm::StrategyId;

use super::{common::OrderId, filled_order::FilledOrder, pending_order::PendingOrder};

#[derive(Debug, Clone, Builder, Getters, PartialEq, Eq)]
#[getset(get = "pub")]
#[builder(setter(prefix = "with"))]
pub struct OrderMeta {
    order_id: OrderId,
    strategy_id: StrategyId,
}

impl OrderMeta {
    pub fn builder() -> OrderMetaBuilder {
        OrderMetaBuilder::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, AsRefStr, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum OrderResult {
    Updated(OrderMeta),
    Cancelled(OrderMeta),
    FilledOrder(FilledOrder),
    PendingOrder(PendingOrder),
}

impl OrderResult {
    pub fn order_id(&self) -> &OrderId {
        match self {
            OrderResult::Updated(o) => &o.order_id,
            OrderResult::Cancelled(o) => &o.order_id,
            OrderResult::FilledOrder(o) => &o.order_id,
            OrderResult::PendingOrder(o) => &o.order_id(),
        }
    }

    pub fn startegy_id(&self) -> StrategyId {
        match self {
            OrderResult::Updated(o) => o.strategy_id,
            OrderResult::Cancelled(o) => o.strategy_id,
            OrderResult::FilledOrder(o) => o.startegy_id(),
            OrderResult::PendingOrder(o) => o.startegy_id(),
        }
    }
}
