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
pub struct OrderDetails {
    pub strategy_id: StrategyId,
    pub quantity: Quantity,
    pub side: Side,
}
