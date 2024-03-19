use crate::{models::security::Security, strategy::algorithm::StrategyId};

use super::{
    common::OrderDetails, limit::Limit, market::Market, one_cancels_others::OneCancelsOthers,
    stop_limit_market::StopLimitMarket,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewOrder {
    Market(Market),
    Limit(Limit),
    StopLimitMarket(StopLimitMarket),
    OCO(OneCancelsOthers),
}

impl NewOrder {
    pub fn startegy_id(&self) -> StrategyId {
        match self {
            NewOrder::Market(o) => o.startegy_id(),
            NewOrder::Limit(o) => o.strategy_id(),
            NewOrder::StopLimitMarket(o) => o.strategy_id(),
            NewOrder::OCO(o) => o.strategy_id(),
        }
    }

    pub fn get_order_details(&self) -> &OrderDetails {
        match self {
            NewOrder::Market(o) => &o.order_details,
            NewOrder::Limit(o) => &o.order_details,
            NewOrder::StopLimitMarket(o) => &o.market.order_details,
            NewOrder::OCO(o) => todo!("I need to think more about this one"),
        }
    }

    pub fn get_security(&self) -> &Security {
        match self {
            NewOrder::Market(o) => &o.security,
            NewOrder::Limit(o) => &o.security,
            NewOrder::StopLimitMarket(o) => &o.market.security,
            NewOrder::OCO(o) => &o.get_security(),
        }
    }
}
