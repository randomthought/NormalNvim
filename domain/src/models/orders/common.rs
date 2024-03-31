use derive_builder::Builder;
use getset::Getters;

use crate::strategy::algorithm::StrategyId;

pub type Quantity = u64;

pub type OrderId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

// https://ibkrguides.com/tws/usersguidebook/ordertypes/time%20in%20force%20for%20orders.htm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    Day,
    GTC,
    OPG,
    IOC,
    GTD,
    DTC,
    // TODO: do you need the below?
    // Fill/Trigger Outside RTH
}

#[derive(Builder, Getters, Debug, Clone, PartialEq, Eq)]
#[getset(get = "pub")]
#[builder(setter(prefix = "with"))]
pub struct OrderDetails {
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub side: Side,
}

impl OrderDetails {
    pub fn builder() -> OrderDetailsBuilder {
        OrderDetailsBuilder::default()
    }
}
