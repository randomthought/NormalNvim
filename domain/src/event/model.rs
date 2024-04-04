use crate::models::price::candle::PriceBar;

#[derive(Debug, Clone)]
pub enum DataEvent {
    Candle(PriceBar),
}
