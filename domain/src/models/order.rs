// TODO: helpful to model more complex order types: https://tlc.thinkorswim.com/center/howToTos/thinkManual/Trade/Order-Entry-Tools/Order-Types
// TODO: heloful for more order types: https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/key-concepts

use std::{time::Duration, u64};

use super::price::Price;
use super::security::Security;
use anyhow::{ensure, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

pub type Quantity = u64;

#[derive(Debug, Clone)]
pub struct Market {
    pub security: Security,
    pub order_details: OrderDetails,
}

impl Market {
    // constructor
    pub fn new(quantity: u64, side: Side, security: Security) -> Self {
        Self {
            security,
            order_details: OrderDetails { quantity, side },
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

#[derive(Debug, Clone)]
pub struct Limit {
    pub price: Price,
    pub security: Security,
    pub times_in_force: TimesInForce,
    pub order_details: OrderDetails,
}

impl Limit {
    pub fn new(
        quantity: u64,
        price: Price,
        side: Side,
        security: Security,
        times_in_force: TimesInForce,
    ) -> Self {
        Self {
            price,
            security,
            times_in_force,
            order_details: OrderDetails { quantity, side },
        }
    }
}

#[derive(Debug, Clone)]
pub struct StopLimitMarket {
    pub stop: Limit,
    pub limit: Limit,
    pub market: Market,
}

impl StopLimitMarket {
    pub fn new(
        security: Security,
        quantity: u64,
        side: Side,
        stop: Price,
        limit: Price,
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

        let times_in_force = TimesInForce::GTC;
        let market_order = Market::new(quantity, side, security.to_owned());
        let profit_limit = Limit::new(quantity, limit, side, security.to_owned(), times_in_force);
        let stop_side = match side {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        };
        let stop_limit = Limit::new(quantity, stop, stop_side, security, times_in_force);

        Ok(Self {
            stop: stop_limit,
            market: market_order,
            limit: profit_limit,
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
            Order::StopLimitMarket(o) => &o.market.security,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub order: Order,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FilledOrder {
    pub security: Security,
    pub order_id: OrderId,
    pub price: Price,
    pub date_time: Duration,
    pub order_details: OrderDetails,
}

impl FilledOrder {
    pub fn new(
        security: Security,
        order_id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
        datetime: Duration,
    ) -> Self {
        let order_details = OrderDetails { quantity, side };
        Self {
            security,
            order_id,
            price,
            date_time: datetime,
            order_details,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OrderResult {
    FilledOrder(FilledOrder),
    PendingOrder(PendingOrder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderDetails {
    pub quantity: Quantity,
    pub side: Side,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPosition {
    pub security: Security,
    pub side: Side,
    pub holding_details: Vec<HoldingDetail>,
}

impl SecurityPosition {
    pub fn get_quantity(&self) -> Quantity {
        self.holding_details
            .iter()
            .fold(0, |acc, next| acc + next.quantity)
    }
}

// TODO: think of a beter name
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoldingDetail {
    pub quantity: Quantity,
    pub price: Price,
}
