// TODO: helpful to model more complex order types: https://tlc.thinkorswim.com/center/howToTos/thinkManual/Trade/Order-Entry-Tools/Order-Types
// TODO: heloful for more order types: https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/key-concepts

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
    quantity: u32,
    side: Side,
    security: Security, // TODO: Consider using lifetime pointer
    times_in_force: TimesInForce,
}

impl Market {
    // constructor
    pub fn new(
        quantity: u32,
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
    quantity: i32,
    price: Price,
    side: Side,
    security: Security, // TODO: Consider using lifetime pointer
    times_in_force: TimesInForce,
}

impl Limit {
    // constructor
    pub fn new(
        quantity: i32,
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
    stop: Price,
    limit: Price,
    side: Side,
    quantity: u64,
    security: Security, // TODO: Consider using lifetime pointer
    times_in_force: TimesInForce,
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

type OrderId = String;

#[derive(Debug, Clone)]
pub enum Order {
    Market(Market),
    Limit(Limit),
    StopLimitMarket(StopLimitMarket),
}

#[derive(Debug, Clone)]
pub struct OrderTicket {
    order_id: OrderId,
    limit: Limit,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FilledOrder {
    pub security: Security,
    pub side: Side,
    pub commission: Price,
    pub price: Price,
    pub quantity: i32,
    pub datetime: i32,
}

#[derive(Debug, Clone)]
pub enum OrderResult {
    FilledOrder(FilledOrder),
    OrderTicket(OrderTicket),
}
