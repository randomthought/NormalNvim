use super::{
    order::{FilledOrder, Order, Side},
    security::Security,
};

pub struct Market {}

pub struct Signal {
    strategy_id: String, // TODO: consider using lifetime pointers
    security: Security,
    side: Side,
    datetime: i32,
    strength: f32,
}

pub enum Event {
    Market(Market),
    Signal(Signal),
    Order(Order),
    FilledOrder(FilledOrder),
}
