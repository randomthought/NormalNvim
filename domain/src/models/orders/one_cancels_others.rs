use derive_builder::Builder;

use crate::{
    models::{price::Price, security::Security},
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
    name = "OneCancelsOthersBuilder",
    build_fn(private, name = "build_seed",)
)]
#[builder(public)]
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
    fn build(&self) -> OneCancelsOthers {
        let orders: Vec<_> = self
            .prices
            .iter()
            .map(|(s, p)| {
                Limit::new(
                    self.quantity,
                    p.to_owned(),
                    s.to_owned(),
                    self.security.to_owned(),
                    self.time_in_force,
                    self.strategy_id,
                )
            })
            .collect();

        OneCancelsOthers { orders }
    }
}

impl OneCancelsOthersBuilder {
    pub fn build(&self) -> Result<OneCancelsOthers, String> {
        let seed = self.build_seed().map_err(|e| e.to_string())?;
        Ok(seed.build())
    }
}
