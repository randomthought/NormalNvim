use std::time::Duration;

use crate::{
    models::{
        order::{NewOrder, Order, PendingOrder},
        price::PriceHistory,
    },
    strategy::algorithm::StrategyId,
};

#[derive(Debug, Clone)]
pub enum Market {
    DataEvent(PriceHistory),
}

#[derive(Debug, Clone)]
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
            Signal::Cancel(s) => s.pending_order.startegy_id(),
            Signal::Liquidate(s) => s,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Entry {
    pub order: NewOrder,
    pub datetime: Duration,
    pub strength: f32,
}

impl Entry {
    pub fn new(order: NewOrder, datetime: Duration, strength: f32) -> Self {
        Self {
            order,
            datetime,
            strength,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Modify {
    pub pending_order: PendingOrder,
    pub datetime: Duration,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Cancel {
    pub pending_order: PendingOrder,
    pub datetime: Duration,
}

#[derive(Debug, Clone)]
// TODO: consuder making pointers to the actual enum data
pub enum Event {
    Market(Market),
    Signal(Signal),
    Order(Order),
}
