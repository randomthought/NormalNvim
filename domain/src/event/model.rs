use crate::models::price::price_bar::PriceBar;

#[derive(Debug, Clone)]
pub enum DataEvent {
    Candle(PriceBar),
}
