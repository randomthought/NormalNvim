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
            Signal::Entry(s) => s.strategy_id,
            Signal::Modify(s) => s.strategy_id,
            Signal::Cancel(s) => s.strategy_id,
            Signal::Liquidate(s) => s,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Entry {
    pub strategy_id: StrategyId,
    pub order: NewOrder,
    pub datetime: Duration,
    pub strength: f32,
}

impl Entry {
    pub fn new(
        strategy_id: StrategyId,
        order: NewOrder,
        datetime: Duration,
        strength: f32,
    ) -> Self {
        Self {
            strategy_id,
            order,
            datetime,
            strength,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Modify {
    pub strategy_id: StrategyId,
    pub pending_order: PendingOrder,
    pub datetime: Duration,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Cancel {
    pub strategy_id: StrategyId,
    pub pending_order: PendingOrder,
    pub datetime: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgoOrder {
    pub strategy_id: StrategyId,
    pub order: Order,
}

#[derive(Debug, Clone)]
pub enum Event {
    Market(Market),
    Signal(Signal),
    AlgoOrder(AlgoOrder),
}
