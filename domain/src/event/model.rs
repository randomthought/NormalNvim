use crate::{
    models::{orders::pending_order::Order, price::PriceHistory},
    strategy::model::signal::Signal,
};

#[derive(Debug, Clone)]
pub enum DataEvent {
    PriceEvent(PriceHistory),
}
