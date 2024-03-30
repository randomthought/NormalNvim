use std::time::Duration;

use derive_builder::Builder;
use derive_getters::Getters;
use strum_macros::{AsRefStr, VariantNames};

use crate::{
    models::orders::{common::OrderId, new_order::NewOrder, pending_order::PendingOrder},
    strategy::algorithm::StrategyId,
};

#[derive(Builder, Debug, Clone, PartialEq, Getters)]
#[builder(setter(prefix = "with"))]
pub struct Entry {
    order: NewOrder,
    datetime: Duration,
    strength: f32,
}

impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }
}

#[derive(Debug, Getters, Builder, Clone, PartialEq, Eq)]
#[builder(public, setter(prefix = "with"))]
#[non_exhaustive]
pub struct Modify {
    pending_order: PendingOrder,
    datetime: Duration,
}

impl Modify {
    pub fn builder() -> ModifyBuilder {
        ModifyBuilder::default()
    }
}

#[derive(Debug, Builder, Getters, Clone, PartialEq, Eq)]
#[builder(public, setter(prefix = "with"))]
#[non_exhaustive]
pub struct Cancel {
    order_id: OrderId,
    strategy_id: StrategyId,
    datetime: Duration,
}

impl Cancel {
    pub fn builder() -> CancelBuilder {
        CancelBuilder::default()
    }
}

#[derive(Debug, Clone, PartialEq, AsRefStr, VariantNames)]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
pub enum Signal {
    Entry(Entry),
    Cancel(Cancel),
    Close(Cancel),
    Modify(Modify),
    Liquidate(StrategyId),
}

impl Signal {
    pub fn strategy_id(&self) -> StrategyId {
        match self {
            Signal::Entry(s) => s.order().startegy_id(),
            Signal::Modify(s) => s.pending_order().startegy_id(),
            Signal::Close(s) => s.strategy_id,
            Signal::Cancel(s) => s.strategy_id,
            Signal::Liquidate(s) => s,
        }
    }
}
