use std::time::Duration;

use derive_builder::Builder;
use getset::Getters;

use crate::{
    models::orders::{common::OrderId, new_order::NewOrder, pending_order::PendingOrder},
    strategy::algorithm::StrategyId,
};

#[derive(Builder, Getters, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Entry {
    #[builder(public, setter(prefix = "with"))]
    #[getset(get)]
    pub order: NewOrder,
    #[builder(public, setter(prefix = "with"))]
    #[getset(get)]
    pub datetime: Duration,
    #[builder(public, setter(prefix = "with"))]
    #[getset(get)]
    pub strength: f32,
}

impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }

    pub fn new(order: NewOrder, datetime: Duration, strength: f32) -> Self {
        Self {
            order,
            datetime,
            strength,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Modify {
    pub pending_order: PendingOrder,
    pub datetime: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Cancel {
    pub order_id: OrderId,
    pub strategy_id: StrategyId,
    pub datetime: Duration,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Signal {
    Entry(Entry),
    Modify(Modify),
    Cancel(Cancel),
    Liquidate(StrategyId),
}

impl Signal {
    pub fn strategy_id(&self) -> StrategyId {
        match self {
            Signal::Entry(s) => s.order.startegy_id(),
            Signal::Modify(s) => s.pending_order.startegy_id(),
            Signal::Cancel(s) => s.strategy_id,
            Signal::Liquidate(s) => s,
        }
    }
}
