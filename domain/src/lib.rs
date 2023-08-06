pub mod model {
    pub mod security {

        type Ticker = String;

        #[derive(Debug)]
        pub enum Exchange {
            // TODO: add list of exchanges
            NASDAQ,
            NYSE,
        }

        #[derive(Debug)]
        pub enum AssetType {
            Equity,
            Forex,
            Future,
            Option,
            Crypto,
        }

        #[derive(Debug)]
        pub struct Security {
            asset_type: AssetType,
            exchange: Exchange,
            ticker: Ticker,
        }

        impl Security {
            pub fn new(ticker: Ticker, exchange: Exchange, asset_type: AssetType) -> Self {
                Security {
                    asset_type,
                    exchange,
                    ticker,
                }
            }
        }
    }

    pub mod price {
        pub type Symbol = String;
        pub type Price = f32;

        #[derive(Debug)]
        pub struct Candle {
            high: Price,
            open: Price,
            low: Price,
            close: Price,
            // The Unix Msec timestamp for the start of the aggregate window.
            time: i32,
            // The trading volume of the symbol in the given time period.
            volume: i32,
        }

        impl Candle {
            pub fn new(
                open: Price,
                high: Price,
                low: Price,
                close: Price,
                volume: i32,
                time: i32,
            ) -> Result<Self, String> {
                if high < low {
                    return Err("High cannot be less than low".to_owned());
                }

                if open > high && open < low {
                    return Err("Open cannot be greater than high or less than low".to_owned());
                }

                if open > close && close < low {
                    return Err("Close cannot be greater than high or less than low".to_owned());
                }

                Ok(Self {
                    open,
                    high,
                    low,
                    close,
                    time,
                    volume,
                })
            }
        }

        pub struct EquityHistory<'a> {
            symbol: Symbol,
            history: Vec<&'a Candle>,
        }
    }

    pub mod order {
        // TODO: helpful to model more complex order types: https://tlc.thinkorswim.com/center/howToTos/thinkManual/Trade/Order-Entry-Tools/Order-Types

        use super::price::Price;
        use super::security::Security;

        #[derive(Debug)]
        pub enum Side {
            Long,
            Short,
        }

        #[derive(Debug)]
        pub struct Market {
            quantity: i32,
            side: Side,
            security: Security,
        }

        #[derive(Debug)]
        pub struct Limit {
            quantity: i32,
            price: Price,
            side: Side,
            security: Security,
        }

        #[derive(Debug)]
        pub struct StopLimitMarket {
            stop: Price,
            market: Market,
            limit: Price,
        }

        impl StopLimitMarket {
            pub fn new(
                security: Security,
                quantity: i32,
                side: Side,
                stop: Price,
                limit: Price,
            ) -> Result<Self, String> {
                if let Side::Long = side {
                    if stop > limit {
                        return Err(
                            "on a long tade, your stop price cannot be greater than your limit"
                                .to_owned(),
                        );
                    }
                }

                if let Side::Short = side {
                    if stop < limit {
                        return Err(
                            "on a short tade, your stop price cannot be less than your limit"
                                .to_owned(),
                        );
                    }
                }

                Ok(Self {
                    stop,
                    limit,
                    market: Market {
                        quantity,
                        side,
                        security,
                    },
                })
            }
        }

        type OrderId = String;

        #[derive(Debug)]
        pub enum Order {
            Market(Market),
            Limit(Limit),
            StopLimitMarket(StopLimitMarket),
        }

        #[derive(Debug)]
        pub struct OrderTicket {
            order_id: OrderId,
            limit: Limit,
        }

        #[derive(Debug)]
        pub struct FilledOrder {
            order_id: OrderId,
            quantity: i32,
            price: Price,
            side: Side,
            security: Security,
        }

        #[derive(Debug)]
        pub enum OrderResult {
            FilledOrder(FilledOrder),
            OrderTicket(OrderTicket),
        }
    }
}

mod strategy {
    use super::model::order::Order;
    use super::model::price::Candle;
    use std::io;
    use std::option::Option;

    pub trait TradeManager {
        fn manage(order: &Order) -> Result<(), io::Error>;
    }

    pub trait Algorithm {
        // TODO: algorthim should probably have a warm up function
        fn on_candle(candle: Candle) -> Result<Option<Order>, io::Error>;
    }
}

mod data {

    pub trait DataProvider {
        fn get();
    }
}

mod broker {
    use super::model::order::{Order, OrderResult, OrderTicket};
    use std::io;

    // Model based on https://developer.tdameritrade.com/account-access/apis
    pub trait Broker {
        fn get() -> Result<Order, io::Error>;
        fn place_order(order: &Order) -> Result<OrderResult, io::Error>;
        fn orders() -> Result<Vec<OrderResult>, io::Error>;
        fn update(orderTicket: &OrderTicket) -> Result<(), io::Error>;
        fn cancel(order: &OrderTicket) -> Result<(), io::Error>;
    }
}
