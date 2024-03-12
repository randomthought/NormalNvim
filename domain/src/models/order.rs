// TODO: helpful to model more complex order types: https://tlc.thinkorswim.com/center/howToTos/thinkManual/Trade/Order-Entry-Tools/Order-Types
// TODO: heloful for more order types: https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/key-concepts

use std::{time::Duration, u64};

use crate::strategy::algorithm::StrategyId;

use super::price::Price;
use super::security::Security;
use color_eyre::eyre::{bail, ensure, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

pub type Quantity = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Market {
    pub security: Security,
    pub order_details: OrderDetails,
}

impl Market {
    // constructor
    pub fn new(
        quantity: Quantity,
        side: Side,
        security: Security,
        strategy_id: StrategyId,
    ) -> Self {
        Self {
            security,
            order_details: OrderDetails {
                quantity,
                side,
                strategy_id,
            },
        }
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}

// https://ibkrguides.com/tws/usersguidebook/ordertypes/time%20in%20force%20for%20orders.htm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        strategy_id: StrategyId,
    ) -> Self {
        Self {
            price,
            security,
            times_in_force,
            order_details: OrderDetails {
                quantity,
                side,
                strategy_id,
            },
        }
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneCancelsOther {
    pub limit_orders: Vec<Limit>,
}

impl OneCancelsOther {
    pub fn new(limit_orders: Vec<Limit>) -> Result<Self> {
        if limit_orders.is_empty() {
            bail!("cannot provide an empty list of limit orders")
        }

        Ok(Self { limit_orders })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopLimitMarket {
    pub one_cancels_other: OneCancelsOther,
    pub market: Market,
}

impl StopLimitMarket {
    pub fn new(
        security: Security,
        quantity: u64,
        side: Side,
        stop: Price,
        limit: Price,
        strategy_id: StrategyId,
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
        let market = Market::new(quantity, side, security.to_owned(), strategy_id);
        let profit_limit = Limit::new(
            quantity,
            limit,
            side,
            security.to_owned(),
            times_in_force,
            strategy_id,
        );
        let stop_side = match side {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        };
        let stop_limit = Limit::new(
            quantity,
            stop,
            stop_side,
            security,
            times_in_force,
            strategy_id,
        );

        let one_cancels_other = OneCancelsOther::new(vec![profit_limit, stop_limit])?;

        Ok(Self {
            market,
            one_cancels_other,
        })
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.market.startegy_id()
    }
}

pub type OrderId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewOrder {
    Market(Market),
    Limit(Limit),
    OCA(OneCancelsOther),
    StopLimitMarket(StopLimitMarket),
}

impl NewOrder {
    pub fn startegy_id(&self) -> StrategyId {
        match self {
            NewOrder::Market(o) => o.startegy_id(),
            NewOrder::Limit(o) => o.strategy_id(),
            NewOrder::OCA(o) => todo!(),
            NewOrder::StopLimitMarket(o) => o.strategy_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOrder {
    pub order_id: OrderId,
    pub order: NewOrder,
}

impl PendingOrder {
    pub fn startegy_id(&self) -> StrategyId {
        self.order.startegy_id()
    }
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
        date_time: Duration,
        strategy_id: StrategyId,
    ) -> Self {
        let order_details = OrderDetails {
            quantity,
            side,
            strategy_id,
        };
        Self {
            security,
            order_id,
            price,
            date_time,
            order_details,
        }
    }

    pub fn startegy_id(&self) -> StrategyId {
        self.order_details.strategy_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderMeta {
    pub order_id: OrderId,
    pub strategy_id: StrategyId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderResult {
    Updated(OrderMeta),
    Cancelled(OrderMeta),
    FilledOrder(FilledOrder),
    PendingOrder(PendingOrder),
}

impl OrderResult {
    pub fn startegy_id(&self) -> StrategyId {
        match self {
            OrderResult::Updated(o) => o.strategy_id,
            OrderResult::Cancelled(o) => o.strategy_id,
            OrderResult::FilledOrder(o) => o.startegy_id(),
            OrderResult::PendingOrder(o) => o.startegy_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderDetails {
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub side: Side,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    NewOrder(NewOrder),
    OrderResult(OrderResult),
}

impl Order {
    pub fn startegy_id(&self) -> StrategyId {
        match self {
            Order::NewOrder(o) => o.startegy_id(),
            Order::OrderResult(o) => o.startegy_id(),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoldingDetail {
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub price: Price,
}
