use super::models::order::Order;
use super::models::price::Candle;
use std::io;
use std::option::Option;

pub trait TradeManager {
    fn manage(order: &Order) -> Result<(), io::Error>;
}

pub trait Algorithm {
    // TODO: algorthim should probably have a warm up function
    fn on_candle(candle: Candle) -> Result<Option<Order>, io::Error>;
}
