use super::models::order::{Order, OrderResult, OrderTicket};
use std::io;

// Model based on https://developer.tdameritrade.com/account-access/apis
pub trait Broker {
    fn get() -> Result<Order, io::Error>;
    fn place_order(order: &Order) -> Result<OrderResult, io::Error>;
    fn orders() -> Result<Vec<OrderResult>, io::Error>;
    fn update(orderTicket: &OrderTicket) -> Result<(), io::Error>;
    fn cancel(order: &OrderTicket) -> Result<(), io::Error>;
}
