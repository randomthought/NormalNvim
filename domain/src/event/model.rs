use std::time::Duration;

use crate::models::{
    order::{FilledOrder, Order, PendingOrder},
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
    pub order: Order,
    pub datetime: Duration,
    pub strength: f32,
}

impl Signal {
    pub fn new(strategy_id: String, order: Order, datetime: Duration, strength: f32) -> Self {
        Self {
            strategy_id,
            order,
            datetime,
            strength,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Market(Market),
    Signal(Signal),
    Order(Order),
    FilledOrder(FilledOrder),
    OrderTicket(PendingOrder),
}
