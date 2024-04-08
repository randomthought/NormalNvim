use derive_builder::Builder;
use getset::Getters;

use crate::{price::common::Price, security::Security, strategy::common::StrategyId};

use super::{
    common::{Side, TimeInForce},
    limit::Limit,
    market::Market,
    one_cancels_others::OneCancelsOthers,
};

#[derive(Debug, Getters, Clone, PartialEq, Eq)]
#[getset(get = "pub")]
pub struct StopLimitMarket {
    one_cancels_others: OneCancelsOthers,
    market: Market,
}

impl StopLimitMarket {
    pub fn builder() -> StopLimitMarketBuilder {
        StopLimitMarketBuilder::default()
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

#[derive(Builder)]
#[builder(
    public,
    name = "StopLimitMarketBuilder",
    build_fn(private, name = "build_seed",)
)]
#[builder(setter(prefix = "with"))]
struct StopLimitMarketSeed {
    security: Security,
    quantity: u64,
    side: Side,
    stop_price: Price,
    limit_price: Price,
    strategy_id: StrategyId,
    #[builder(default = "TimeInForce::GTC")]
    times_in_force: TimeInForce,
}

impl StopLimitMarketSeed {
    fn build(&self) -> Result<StopLimitMarket, StopLimitMarketBuilderError> {
        let market = Market::builder()
            .with_security(self.security.to_owned())
            .with_strategy_id(self.strategy_id)
            .with_quantity(self.quantity)
            .with_side(self.side)
            .build()
            .map_err(|e| e.to_string())?;

        let stop_side = match self.side {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        };

        let one_cancels_others = OneCancelsOthers::builder()
            .with_quantity(self.quantity)
            .with_security(self.security.to_owned())
            .with_strategy_id(self.strategy_id)
            .with_time_in_force(self.times_in_force)
            .add_limit(stop_side, self.stop_price)
            .add_limit(self.side, self.limit_price)
            .build()
            .map_err(|e| e.to_string())?;

        Ok(StopLimitMarket {
            market,
            one_cancels_others,
        })
    }
}

impl StopLimitMarketBuilder {
    pub fn build(&self) -> Result<StopLimitMarket, StopLimitMarketBuilderError> {
        let seed = self.build_seed()?;
        seed.build()
    }

    fn validate(&self) -> Result<(), String> {
        let Some(limit_side) = self.side else {
            return Ok(());
        };

        let Some(stop_price) = self.stop_price else {
            return Ok(());
        };
        let Some(limit_price) = self.limit_price else {
            return Ok(());
        };

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

        Ok(())
    }
}
