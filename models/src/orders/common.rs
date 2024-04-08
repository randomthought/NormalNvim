use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::strategy::common::StrategyId;

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

#[derive(Builder, CopyGetters, Debug, Clone, PartialEq, Eq)]
#[getset(get_copy = "pub")]
#[builder(setter(prefix = "with"))]
pub struct OrderDetails {
    strategy_id: StrategyId,
    quantity: Quantity,
    side: Side,
}

impl OrderDetails {
    pub fn builder() -> OrderDetailsBuilder {
        OrderDetailsBuilder::default()
    }
}
