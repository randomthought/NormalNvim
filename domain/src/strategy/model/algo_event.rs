use crate::{event::model::Market, models::orders::order_result::OrderResult};

pub enum AlgoEvent<'a> {
    Market(&'a Market),
    OrderResult(&'a OrderResult),
}
