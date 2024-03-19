use crate::{event::model::Market, models::orders::order_result::OrderResult};

#[derive(Clone)]
pub enum AlgoEvent {
    Market(Market),
    OrderResult(OrderResult),
}
