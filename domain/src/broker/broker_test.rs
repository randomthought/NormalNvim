use core::panic;
use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    broker::broker::Broker,
    data::QouteProvider,
    event::{event::EventProducer, model::Event},
    models::{
        order::{
            self, FilledOrder, HoldingDetail, Market, Order, OrderResult, PendingOrder,
            SecurityPosition, Side,
        },
        price::{Price, Quote},
        security::{AssetType, Exchange, Security},
    },
    order::{Account, OrderManager, OrderReader},
};

use anyhow::{Ok, Result};

struct Setup {
    pub security: Security,
    pub price: Price,
}

impl Setup {
    pub fn new() -> Self {
        let security = Security::new(AssetType::Equity, Exchange::NYSE, "GE".into());
        Self {
            security,
            price: Decimal::new(1000, 0),
        }
    }
}

struct Stub;

#[cfg(test)]
#[async_trait]
impl QouteProvider for Stub {
    async fn get_quote(&self, security: &Security) -> Result<Quote> {
        let quote = Quote {
            security: security.to_owned(),
            bid: Decimal::new(1000, 0),
            ask: Decimal::new(1000, 0),
            ask_size: 0,
            bid_size: 0,
            timestamp: Duration::new(5, 0),
        };

        Ok(quote)
    }
}

#[async_trait]
impl EventProducer for Stub {
    async fn produce(&self, event: Event) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn get_account_balance() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());

    let results = broker.get_account_balance().await.unwrap();

    assert_eq!(balance, results)
}

#[tokio::test]
async fn close_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());

    let quantity_1 = 10;
    let market_order_1 = Order::Market(Market::new(
        quantity_1,
        Side::Long,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_1).await.unwrap();
    let quantity_2 = 10;
    let market_order_2 = Order::Market(Market::new(
        quantity_2,
        Side::Long,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_2).await.unwrap();
    let market_order_3 = Order::Market(Market::new(
        quantity_1 + quantity_2,
        Side::Short,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_3).await.unwrap();

    let result = broker.get_positions().await.unwrap();

    assert!(result.is_empty(), "the shouldn't be any open positions");
}

#[tokio::test]
async fn flip_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());

    let quantity_1 = 10;
    let market_order_1 = Order::Market(Market::new(
        quantity_1,
        Side::Long,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_1).await.unwrap();
    let quantity_2 = 10;
    let market_order_2 = Order::Market(Market::new(
        quantity_2,
        Side::Long,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_2).await.unwrap();
    let quantity_3 = 40;
    let market_order_3 = Order::Market(Market::new(
        quantity_3,
        Side::Short,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_3).await.unwrap();

    let result = broker.get_positions().await.unwrap();

    let expected = vec![SecurityPosition {
        security: setup.security.to_owned(),
        side: Side::Short,
        holding_details: vec![HoldingDetail {
            quantity: 20,
            price: setup.price,
        }],
    }];

    assert_eq!(expected, result)
}

#[tokio::test]
async fn get_balance_after_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let market = Market::new(quantity, side, setup.security.to_owned());
    let market_order = Order::Market(market);
    broker.place_order(&market_order).await.unwrap();

    let result = broker.get_account_balance().await.unwrap();

    let expected = Decimal::new(90_000, 0);

    assert_eq!(expected, result);
}

#[tokio::test]
async fn get_balance_after_profit() {
    todo!()
}

#[tokio::test]
async fn get_balance_after_loss() {
    todo!()
}

#[tokio::test]
async fn get_positions() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let market = Market::new(quantity, side, setup.security.to_owned());
    let market_order = Order::Market(market);
    broker.place_order(&market_order).await.unwrap();

    let expected = vec![SecurityPosition {
        security: setup.security.to_owned(),
        side,
        holding_details: vec![HoldingDetail {
            quantity,
            price: setup.price,
        }],
    }];

    let result = broker.get_positions().await.unwrap();

    assert_eq!(expected, result);
}

#[tokio::test]
async fn get_pending_orders() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let pending_order = Order::Limit(order::Limit::new(
        quantity,
        setup.price,
        side,
        setup.security.to_owned(),
        order::TimesInForce::GTC,
    ));
    broker.place_order(&pending_order).await.unwrap();

    let results = broker.pending_orders().await.unwrap();

    assert!(
        results.is_empty(),
        "pending order was inserted but none is returned"
    )
}

#[tokio::test]
async fn cancel_pending_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let limit_order = Order::Limit(order::Limit::new(
        quantity,
        setup.price,
        side,
        setup.security.to_owned(),
        order::TimesInForce::GTC,
    ));

    let OrderResult::FilledOrder(filled_order) = broker.place_order(&limit_order).await.unwrap() else {
        panic!("must get a filled result")
    };

    let pending_order = PendingOrder {
        order_id: filled_order.order_id.to_owned(),
        order: limit_order.to_owned(),
    };
    broker.cancel(&pending_order).await.unwrap();

    let pending_orders = broker.pending_orders().await.unwrap();

    assert!(
        pending_orders.is_empty(),
        "failed to remove canceled pending order"
    )
}

#[tokio::test]
async fn update_pending_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub {});
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let limit_order = Order::Limit(order::Limit::new(
        quantity,
        setup.price,
        side,
        setup.security.to_owned(),
        order::TimesInForce::GTC,
    ));

    todo!()
}

#[tokio::test]
async fn close_existing_trade_on_low_balance() {
    // Close existing trade when balance is 0
    todo!()
}
