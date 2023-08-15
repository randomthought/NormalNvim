use super::{
    order::{FilledOrder, Order, Side},
    price::PriceHistory,
    security::Security,
};

#[derive(Debug, Clone)]
pub enum Market {
    DataEvent(PriceHistory),
}

#[derive(Debug, Clone)]
pub struct Signal {
    strategy_id: String,
    security: Security,
    side: Side,
    datetime: i32,
    strength: f32,
}

#[derive(Debug, Clone)]
pub enum Event {
    Market(Market),
    Signal(Signal),
    Order(Order),
    FilledOrder(FilledOrder),
}
