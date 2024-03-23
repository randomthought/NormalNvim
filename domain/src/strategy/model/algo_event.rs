use crate::{event::model::DataEvent, models::orders::order_result::OrderResult};
use strum_macros::AsRefStr;
use strum_macros::VariantNames;

#[derive(Clone, AsRefStr, VariantNames)]
pub enum AlgoEvent {
    DataEvent(DataEvent),
    OrderResult(OrderResult),
}
