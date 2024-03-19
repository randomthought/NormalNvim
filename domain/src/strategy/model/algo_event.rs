use crate::{event::model::DataEvent, models::orders::order_result::OrderResult};

#[derive(Clone)]
pub enum AlgoEvent {
    DataEvent(DataEvent),
    OrderResult(OrderResult),
}
