use crate::{event::model::DataEvent, models::orders::order_result::OrderResult};
use strum_macros::AsRefStr;
use strum_macros::VariantNames;

#[derive(Clone, AsRefStr, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum AlgoEvent {
    DataEvent(DataEvent),
    OrderResult(OrderResult),
}
