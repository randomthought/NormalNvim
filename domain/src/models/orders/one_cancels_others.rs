use crate::{
    models::{price::Price, security::Security},
    strategy::algorithm::StrategyId,
};

use super::{
    common::{Quantity, Side, TimesInForce},
    limit::Limit,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneCancelsOthers {
    pub orders: Vec<Limit>,
    security: Security,
    quantity: Quantity,
    strategy_id: StrategyId,
}

impl OneCancelsOthers {
    pub fn builder() -> OneCancelsOthersBuilder {
        OneCancelsOthersBuilder::default()
    }

    pub fn get_quantity(&self) -> Quantity {
        self.quantity
    }

    pub fn get_security(&self) -> &Security {
        &self.security
    }

    pub fn strategy_id(&self) -> StrategyId {
        &self.strategy_id
    }
}

#[derive(Default)]
pub struct OneCancelsOthersBuilder {
    strategy_id: Option<StrategyId>,
    times_in_force: Option<TimesInForce>,
    security: Option<Security>,
    quantity: Option<u64>,
    prices: Vec<(Side, Price)>,
}

impl OneCancelsOthersBuilder {
    pub fn new() -> Self {
        OneCancelsOthersBuilder {
            strategy_id: None,
            times_in_force: None,
            security: None,
            quantity: None,
            prices: vec![],
        }
    }

    pub fn with_time_in_force(mut self, times_in_force: TimesInForce) -> Self {
        self.times_in_force = Some(times_in_force);
        self
    }
    pub fn with_strategy_id(mut self, strategy_id: StrategyId) -> Self {
        self.strategy_id = Some(strategy_id);
        self
    }
    pub fn with_security(mut self, security: Security) -> Self {
        self.security = Some(security);
        self
    }

    pub fn with_quantity(mut self, quantity: u64) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn add_limit(mut self, side: Side, price: Price) -> Self {
        self.prices.push((side, price));
        self
    }

    pub fn build(self) -> Result<OneCancelsOthers, String> {
        if self.prices.is_empty() {
            return Err("prices cannot be empty".to_string());
        }

        let security = self.security.ok_or("security is required".to_string())?;
        let quantity = self.quantity.ok_or("quantity is required".to_string())?;
        if quantity == 0 {
            return Err("quantity cannot be zero".to_string());
        }

        let strategy_id = self
            .strategy_id
            .ok_or("strategy_id is required".to_string())?;
        let times_in_force = self
            .times_in_force
            .ok_or("times_in_force is required".to_string())?;

        let orders: Vec<_> = self
            .prices
            .iter()
            .map(|(s, p)| {
                Limit::new(
                    quantity,
                    p.to_owned(),
                    s.to_owned(),
                    security.to_owned(),
                    times_in_force,
                    strategy_id,
                )
            })
            .collect();

        Ok(OneCancelsOthers {
            strategy_id,
            security,
            quantity,
            orders,
        })
    }
}
