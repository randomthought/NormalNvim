use anyhow::{ensure, Result};

use crate::models::{
    order::{FilledOrder, Order, OrderTicket, Side, TimesInForce},
    price::{Price, PriceHistory},
    security::Security,
};

#[derive(Debug, Clone)]
pub enum Market {
    DataEvent(PriceHistory),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Signal {
    // TODO: create constructor
    pub strategy_id: String,
    pub security: Security,
    pub stop: Price,
    pub limit: Price,
    pub side: Side,
    pub datetime: i32,
    pub strength: f32,
    pub times_in_force: TimesInForce,
}

impl Signal {
    pub fn new(
        strategy_id: String,
        security: Security,
        stop: Price,
        limit: Price,
        side: Side,
        times_in_force: TimesInForce,
        datetime: i32,
        strength: f32,
    ) -> Result<Self> {
        match side {
            Side::Long => {
                ensure!(
                    limit > stop,
                    "limit has to be greater than the stop price on a long signal"
                );
            }

            Side::Short => {
                ensure!(
                    limit < stop,
                    "limit has to be greater than the stop price on a long signal"
                );
            }
        }

        Ok(Signal {
            strategy_id,
            security,
            stop,
            limit,
            side,
            datetime,
            strength,
            times_in_force,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Market(Market),
    Signal(Signal),
    Order(Order),
    FilledOrder(FilledOrder),
    OrderTicket(OrderTicket),
}
