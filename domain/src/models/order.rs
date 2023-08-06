// TODO: helpful to model more complex order types: https://tlc.thinkorswim.com/center/howToTos/thinkManual/Trade/Order-Entry-Tools/Order-Types

use super::price::Price;
use super::security::Security;

#[derive(Debug)]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug)]
pub struct Market {
    quantity: i32,
    side: Side,
    security: Security, // TODO: Consider using lifetime pointer
    times_in_force: TimesInForce,
}

impl Market {
    // constructor
    pub fn new(
        quantity: i32,
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
#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
pub struct StopLimitMarket {
    stop: Price,
    limit: Price,
    side: Side,
    quantity: i32,
    security: Security, // TODO: Consider using lifetime pointer
    times_in_force: TimesInForce,
}

impl StopLimitMarket {
    pub fn new(
        security: Security,
        quantity: i32,
        side: Side,
        stop: Price,
        limit: Price,
        times_in_force: TimesInForce,
    ) -> Result<Self, String> {
        if let Side::Long = side {
            if stop > limit {
                return Err(
                    "on a long tade, your stop price cannot be greater than your limit".to_owned(),
                );
            }
        }

        if let Side::Short = side {
            if stop < limit {
                return Err(
                    "on a short tade, your stop price cannot be less than your limit".to_owned(),
                );
            }
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

#[derive(Debug)]
pub enum Order {
    Market(Market),
    Limit(Limit),
    StopLimitMarket(StopLimitMarket),
}

#[derive(Debug)]
pub struct OrderTicket {
    order_id: OrderId,
    limit: Limit,
}

#[derive(Debug)]
pub struct FilledOrder {}

#[derive(Debug)]
pub enum OrderResult {
    FilledOrder(FilledOrder),
    OrderTicket(OrderTicket),
}
