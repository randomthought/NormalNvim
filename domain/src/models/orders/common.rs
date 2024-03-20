use derive_builder::Builder;

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

#[derive(Builder, Debug, Clone, PartialEq, Eq)]
pub struct OrderDetails {
    #[builder(setter(prefix = "with"))]
    pub strategy_id: StrategyId,
    #[builder(setter(prefix = "with"))]
    pub quantity: Quantity,
    #[builder(setter(prefix = "with"))]
    pub side: Side,
}

impl OrderDetails {
    pub fn builder() -> OrderDetailsBuilder {
        OrderDetailsBuilder::default()
    }
}
