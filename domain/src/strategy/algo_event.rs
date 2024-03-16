use crate::{event::model::Market, models::order::OrderResult};

pub enum AlgoEvent<'a> {
    Market(&'a Market),
    OrderResult(&'a OrderResult),
}
