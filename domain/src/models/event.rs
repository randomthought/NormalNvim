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
    // TODO: create constructor
    pub strategy_id: String,
    pub security: Security,
    pub side: Side,
    pub datetime: i32,
    pub strength: f32,
}

#[derive(Debug, Clone)]
pub enum Event {
    Market(Market),
    Signal(Signal),
    Order(Order),
    FilledOrder(FilledOrder),
}
