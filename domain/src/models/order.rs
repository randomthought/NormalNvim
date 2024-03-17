// TODO: helpful to model more complex order types: https://tlc.thinkorswim.com/center/howToTos/thinkManual/Trade/Order-Entry-Tools/Order-Types
// TODO: heloful for more order types: https://www.quantconnect.com/docs/v2/writing-algorithms/trading-and-orders/key-concepts

use std::{time::Duration, u64};

use crate::strategy::algorithm::StrategyId;

use super::price::Price;
use super::security::Security;

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
pub struct StopLimitMarket {
    pub one_cancels_others: OneCancelsOthers,
    pub market: Market,
}

impl StopLimitMarket {
    pub fn new(
        security: Security,
        quantity: u64,
        limit_side: Side,
        stop_price: Price,
        limit_price: Price,
        strategy_id: StrategyId,
    ) -> Result<Self, String> {
        if let Side::Long = limit_side {
            if stop_price > limit_price {
                return Err(
                    "on a long tade, your stop price cannot be greater than your limit".into(),
                );
            }
        }

        if let Side::Short = limit_side {
            if stop_price < limit_price {
                return Err(
                    "on a short tade, your stop price cannot be less than your limit".into(),
                );
            }
        }

        let times_in_force = TimesInForce::GTC;
        let market = Market::new(quantity, limit_side, security.to_owned(), strategy_id);
        let stop_side = match limit_side {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        };

        let one_cancels_others = OneCancelsOthers::builder()
            .with_quantity(quantity)
            .with_security(security.to_owned())
            .with_strategy_id(strategy_id)
            .with_time_in_force(times_in_force)
            .add_limit(stop_side, stop_price)
            .add_limit(limit_side, limit_price)
            .build()
            .unwrap();

        Ok(Self {
            market,
            one_cancels_others,
        })
    }

    pub fn strategy_id(&self) -> StrategyId {
        self.market.startegy_id()
    }

    pub fn get_limit(&self) -> &Limit {
        self.one_cancels_others.orders.last().unwrap()
    }

    pub fn get_stop(&self) -> &Limit {
        self.one_cancels_others.orders.first().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneCancelsOthers {
    pub orders: Vec<Limit>,
    security: Security,
    quantity: Quantity,
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
            security,
            quantity,
            orders,
        })
    }
}

pub type OrderId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewOrder {
    Market(Market),
    Limit(Limit),
    StopLimitMarket(StopLimitMarket),
    OCO(OneCancelsOthers),
}

impl NewOrder {
    pub fn startegy_id(&self) -> StrategyId {
        match self {
            NewOrder::Market(o) => o.startegy_id(),
            NewOrder::Limit(o) => o.strategy_id(),
            NewOrder::StopLimitMarket(o) => o.strategy_id(),
            NewOrder::OCO(_) => todo!(),
        }
    }

    pub fn get_order_details(&self) -> &OrderDetails {
        todo!()
    }

    pub fn get_security(&self) -> &Security {
        todo!()
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
