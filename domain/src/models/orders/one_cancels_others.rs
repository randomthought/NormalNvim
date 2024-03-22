use derive_builder::Builder;

use crate::{
    models::{price::common::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::{
    common::{Quantity, Side, TimeInForce},
    limit::Limit,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneCancelsOthers {
    pub orders: Vec<Limit>,
}

impl OneCancelsOthers {
    pub fn builder() -> OneCancelsOthersBuilder {
        OneCancelsOthersBuilder::default()
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.orders.first().unwrap().strategy_id()
    }

    pub fn get_security(&self) -> &Security {
        &self.orders.first().unwrap().security
    }
}

#[derive(Builder)]
#[builder(
    public,
    name = "OneCancelsOthersBuilder",
    build_fn(private, name = "build_seed",)
)]
struct OneCancelsOthersSeed {
    #[builder(private)]
    prices: Vec<(Side, Price)>,
    #[builder(setter(prefix = "with"))]
    time_in_force: TimeInForce,
    #[builder(setter(prefix = "with"))]
    quantity: Quantity,
    #[builder(setter(prefix = "with"))]
    strategy_id: StrategyId,
    #[builder(setter(prefix = "with"))]
    security: Security,
}

impl OneCancelsOthersBuilder {
    pub fn add_limit(&mut self, side: Side, price: Price) -> &mut Self {
        let item = (side, price);
        let Some(prices) = self.prices.as_mut() else {
            self.prices = Some(vec![item]);
            return self;
        };

        prices.push(item);

        self
    }
}

impl OneCancelsOthersSeed {
    fn build(&self) -> Result<OneCancelsOthers, OneCancelsOthersBuilderError> {
        let mut orders: Vec<Limit> = vec![];

        for (s, p) in self.prices.iter() {
            let limit = Limit::builder()
                .with_quantity(self.quantity)
                .with_strategy_id(self.strategy_id)
                .with_side(s.to_owned())
                .with_price(p.to_owned())
                .with_times_in_force(self.time_in_force)
                .with_security(self.security.to_owned())
                .build()
                .map_err(|e| e.to_string())?;

            orders.push(limit);
        }

        Ok(OneCancelsOthers { orders })
    }
}

impl OneCancelsOthersBuilder {
    pub fn build(&self) -> Result<OneCancelsOthers, OneCancelsOthersBuilderError> {
        let seed = self.build_seed()?;
        seed.build()
    }
}
