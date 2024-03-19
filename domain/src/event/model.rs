use crate::{
    models::{orders::pending_order::Order, price::PriceHistory},
    strategy::model::signal::Signal,
};

#[derive(Debug, Clone)]
pub enum DataEvent {
    PriceEvent(PriceHistory),
}

#[derive(Debug, Clone)]
// TODO: consuder making pointers to the actual enum data
pub enum Event {
    Market(DataEvent),
    Signal(Signal),
    Order(Order),
}
