use crate::price::price_bar::PriceBar;

#[derive(Debug, Clone)]
pub enum DataEvent {
    PriceBar(PriceBar),
}
