use super::{
    order::{FilledOrder, Order, Side},
    price::PriceHistory,
    security::Security,
};

#[derive(Debug, Clone, Copy)]
pub enum Market {
    DataEvent(PriceHistory),
}

#[derive(Debug, Clone, Copy)]
pub struct Signal {
    // TODO: Find a way to make this work
    // strategy_id: String, // TODO: consider using lifetime pointers
    security: Security,
    side: Side,
    datetime: i32,
    strength: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Market(Market),
    Signal(Signal),
    Order(Order),
    FilledOrder(FilledOrder),
}
