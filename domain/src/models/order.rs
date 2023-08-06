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
    security: Security,
}

impl Market {
    // constructor
    pub fn new(quantity: i32, side: Side, security: Security) -> Self {
        Self {
            quantity,
            side,
            security,
        }
    }
}

#[derive(Debug)]
pub struct Limit {
    quantity: i32,
    price: Price,
    side: Side,
    security: Security,
}

impl Limit {
    // constructor
    pub fn new(quantity: i32, price: Price, side: Side, security: Security) -> Self {
        Self {
            quantity,
            price,
            side,
            security,
        }
    }
}

#[derive(Debug)]
pub struct StopLimitMarket {
    stop: Price,
    market: Market,
    limit: Price,
}

impl StopLimitMarket {
    pub fn new(
        security: Security,
        quantity: i32,
        side: Side,
        stop: Price,
        limit: Price,
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
            market: Market {
                quantity,
                side,
                security,
            },
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
