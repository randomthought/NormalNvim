use crate::{
    models::{price::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::{
    common::{Side, TimesInForce},
    limit::Limit,
    market::Market,
    one_cancels_others::OneCancelsOthers,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopLimitMarket {
    pub one_cancels_others: OneCancelsOthers,
    pub market: Market,
}

impl StopLimitMarket {
    pub fn new(
        security: Security,
        quantity: u64,
        limit_side: Side,
        stop_price: Price,
        limit_price: Price,
        strategy_id: StrategyId,
    ) -> Result<Self, String> {
        if let Side::Long = limit_side {
            if stop_price > limit_price {
                return Err(
                    "on a long tade, your stop price cannot be greater than your limit".into(),
                );
            }
        }

        if let Side::Short = limit_side {
            if stop_price < limit_price {
                return Err(
                    "on a short tade, your stop price cannot be less than your limit".into(),
                );
            }
        }

        let times_in_force = TimesInForce::GTC;
        let market = Market::new(quantity, limit_side, security.to_owned(), strategy_id);
        let stop_side = match limit_side {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        };

        let one_cancels_others = OneCancelsOthers::builder()
            .with_quantity(quantity)
            .with_security(security.to_owned())
            .with_strategy_id(strategy_id)
            .with_time_in_force(times_in_force)
            .add_limit(stop_side, stop_price)
            .add_limit(limit_side, limit_price)
            .build()
            .unwrap();

        Ok(Self {
            market,
            one_cancels_others,
        })
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.market.startegy_id()
    }

    pub fn get_limit(&self) -> &Limit {
        self.one_cancels_others.orders.last().unwrap()
    }

    pub fn get_stop(&self) -> &Limit {
        self.one_cancels_others.orders.first().unwrap()
    }
}
