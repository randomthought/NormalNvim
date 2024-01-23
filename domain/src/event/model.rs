use std::time::Duration;

use crate::models::{
    order::{NewOrder, Order},
    price::PriceHistory,
};

#[derive(Debug, Clone)]
pub enum Market {
    DataEvent(PriceHistory),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Signal {
    pub strategy_id: String,
    pub order: NewOrder,
    pub datetime: Duration,
    pub strength: f32,
}

impl Signal {
    pub fn new(strategy_id: String, order: NewOrder, datetime: Duration, strength: f32) -> Self {
        Self {
            strategy_id,
            order,
            datetime,
            strength,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgoOrder {
    pub strategy_id: String,
    pub order: Order,
}

#[derive(Debug, Clone)]
pub enum Event {
    Market(Market),
    Signal(Signal),
    AlgoOrder(AlgoOrder),
}
