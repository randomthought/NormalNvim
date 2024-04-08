use strum_macros::AsRefStr;
use strum_macros::VariantNames;

use crate::event::DataEvent;
use crate::orders::order_result::OrderResult;

#[derive(Clone, AsRefStr, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum AlgoEvent {
    DataEvent(DataEvent),
    OrderResult(OrderResult),
}
