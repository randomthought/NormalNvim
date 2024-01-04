// TODO: helpful to model more complex order types: https://tlc.thinkorswim.com/center/howToTos/thinkManual/Trade/Order-Entry-Tools/Order-Types
// TODO: heloful for more order types: https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/key-concepts

use std::time::Duration;

use super::price::Price;
use super::security::Security;
use anyhow::{ensure, Result};

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct Market {
    pub quantity: u64,
    pub side: Side,
    pub security: Security, // TODO: Consider using lifetime pointer
    pub times_in_force: TimesInForce,
}

impl Market {
    // constructor
    pub fn new(
        quantity: u64,
        side: Side,
        security: Security,
        times_in_force: TimesInForce,
    ) -> Self {
        Self {
            quantity,
            side,
            security,
            times_in_force,
        }
    }
}

// https://ibkrguides.com/tws/usersguidebook/ordertypes/time%20in%20force%20for%20orders.htm
#[derive(Debug, Clone, Copy)]
pub enum TimesInForce {
    Day,
    GTC,
    OPG,
    IOC,
    GTD,
    DTC,
    // TODO: do you need the below?
    // Fill/Trigger Outside RTH
}

// TODO: Add order durtation example, day order
#[derive(Debug, Clone)]
pub struct Limit {
    pub quantity: u64,
    pub price: Price,
    pub side: Side,
    pub security: Security, // TODO: Consider using lifetime pointer
    pub times_in_force: TimesInForce,
}

impl Limit {
    // constructor
    pub fn new(
        quantity: u64,
        price: Price,
        side: Side,
        security: Security,
        times_in_force: TimesInForce,
    ) -> Self {
        Self {
            quantity,
            price,
            side,
            security,
            times_in_force,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StopLimitMarket {
    pub stop: Price,
    pub limit: Price,
    pub side: Side,
    pub quantity: u64,
    pub security: Security, // TODO: Consider using lifetime pointer
    pub times_in_force: TimesInForce,
}

impl StopLimitMarket {
    pub fn new(
        security: Security,
        quantity: u64,
        side: Side,
        stop: Price,
        limit: Price,
        times_in_force: TimesInForce,
    ) -> Result<Self> {
        if let Side::Long = side {
            ensure!(
                stop < limit,
                "on a long tade, your stop price cannot be greater than your limit"
            );
        }

        if let Side::Short = side {
            ensure!(
                stop > limit,
                "on a short tade, your stop price cannot be less than your limit"
            );
        }

        Ok(Self {
            stop,
            limit,
            side,
            quantity,
            security,
            times_in_force,
        })
    }
}

pub type OrderId = String;

#[derive(Debug, Clone)]
pub enum Order {
    Market(Market),
    Limit(Limit),
    StopLimitMarket(StopLimitMarket),
}

impl Order {
    pub fn get_security(&self) -> &Security {
        match self {
            Order::Market(o) => &o.security,
            Order::Limit(o) => &o.security,
            Order::StopLimitMarket(o) => &o.security,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub order: Order,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FilledOrder {
    pub order_id: OrderId,
    pub security: Security,
    pub side: Side,
    pub commission: Price,
    pub price: Price,
    pub quantity: u64,
    pub datetime: Duration,
}

#[derive(Debug, Clone)]
pub enum OrderResult {
    FilledOrder(FilledOrder),
    PendingOrder(PendingOrder),
}
