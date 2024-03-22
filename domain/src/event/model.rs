use crate::models::price::candle::Candle;

#[derive(Debug, Clone)]
pub enum DataEvent {
    Candle(Candle),
}
