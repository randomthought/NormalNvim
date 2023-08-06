use domain::models::{
    order::{Side, StopLimitMarket},
    security::{AssetType, Exchange, Security},
};

fn main() {
    let sec = Security::new("NYSE".to_owned(), Exchange::NYSE, AssetType::Equity);
    let stm = StopLimitMarket::new(sec, 2, Side::Long, 2.3, 2.6).unwrap();

    println!("Limit order: {stm:?}");
}
