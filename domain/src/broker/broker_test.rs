use core::panic;
use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    ops::Add,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use crossbeam::epoch::Pointable;
use futures_util::lock::Mutex;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tokio::sync::RwLock;

use crate::{
    broker::broker::Broker,
    data::QouteProvider,
    event::{event::EventProducer, model::Event},
    models::{
        order::{
            self, FilledOrder, HoldingDetail, Market, OneCancelsOther, Order, OrderResult,
            PendingOrder, SecurityPosition, Side, StopLimitMarket,
        },
        price::{self, Price, Quote},
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
        let price = Decimal::new(1000, 0);
        Self {
            security,
            price: Decimal::new(1000, 0),
        }
    }
}

struct Stub {
    price: RwLock<Price>,
}

impl Stub {
    pub fn new() -> Self {
        Self {
            price: RwLock::new(Decimal::new(1000, 0)),
        }
    }
    pub async fn add_to_price(&self, price: Price) {
        let mut p = self.price.write().await;
        *p = *p + price
    }
}

#[cfg(test)]
#[async_trait]
impl QouteProvider for Stub {
    async fn get_quote(&self, security: &Security) -> Result<Quote> {
        let price = self.price.read().await;

        let quote = Quote {
            security: security.to_owned(),
            bid: *price,
            ask: *price,
            ask_size: 0,
            bid_size: 0,
            timestamp: Duration::new(5, 0),
        };

        Ok(quote)
    }
}

#[async_trait]
impl EventProducer for Stub {
    async fn produce(&self, _: Event) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn get_account_balance() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());

    let results = broker.get_account_balance().await.unwrap();

    assert_eq!(balance, results)
}

#[tokio::test]
async fn close_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
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

    let stub = Arc::new(Stub::new());
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

    let stub = Arc::new(Stub::new());
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
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let balance_before_trade = broker.get_account_balance().await.unwrap();
    let market = Market::new(quantity, side, setup.security.to_owned());
    let market_order = Order::Market(market);
    let _ = broker.place_order(&market_order).await.unwrap();
    stub.add_to_price(Decimal::new(10, 0)).await;
    let market_order_close = Order::Market(Market::new(
        quantity,
        Side::Short,
        setup.security.to_owned(),
    ));
    let _ = broker.place_order(&market_order_close).await.unwrap();
    let balance_after_trade = broker.get_account_balance().await.unwrap();

    assert!(
        balance_after_trade > balance_before_trade,
        "profit should have been made"
    );

    let expected = Decimal::new(101_000, 0);
    assert_eq!(balance_after_trade, expected)
}

#[tokio::test]
async fn get_balance_after_loss() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let balance_before_trade = broker.get_account_balance().await.unwrap();
    let market = Market::new(quantity, side, setup.security.to_owned());
    let market_order = Order::Market(market);
    let _ = broker.place_order(&market_order).await.unwrap();
    stub.add_to_price(Decimal::new(-10, 0)).await;
    let market_order_close = Order::Market(Market::new(
        quantity,
        Side::Short,
        setup.security.to_owned(),
    ));
    let _ = broker.place_order(&market_order_close).await.unwrap();
    let balance_after_trade = broker.get_account_balance().await.unwrap();

    assert!(
        balance_after_trade < balance_before_trade,
        "loss should have been made"
    );

    let expected = Decimal::new(99_900, 0);
    assert_eq!(balance_after_trade, expected)
}

#[tokio::test]
async fn get_positions() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
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

    let stub = Arc::new(Stub::new());
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

    let results = broker.get_pending_orders().await.unwrap();

    assert!(
        results.is_empty(),
        "pending order was inserted but none is returned"
    )
}

#[tokio::test]
async fn insert_market_stop_limit_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());

    let quantity = 10;
    let side = Side::Long;
    let limit_price = Decimal::new(100, 0);
    let stop_price = Decimal::new(90, 0);
    let stop_limit_market = StopLimitMarket::new(
        setup.security.to_owned(),
        quantity,
        side,
        stop_price,
        limit_price,
    )
    .unwrap();
    let order = Order::StopLimitMarket(stop_limit_market);
    let order_result = broker.place_order(&order).await.unwrap();

    let expected_1 = vec![SecurityPosition {
        security: setup.security.to_owned(),
        side,
        holding_details: vec![HoldingDetail {
            quantity,
            price: setup.price,
        }],
    }];

    let results_1 = broker.get_positions().await.unwrap();

    assert_eq!(expected_1, results_1);

    let p = PendingOrder {
        order_id: "todo".to_owned(),
        order: order.to_owned(),
    };

    let expected_2: Vec<OrderResult> = vec![OrderResult::PendingOrder(PendingOrder {
        order_id: "todo".to_owned(),
        order: order.to_owned(),
    })];

    let Some(OrderResult::PendingOrder(pending_order)) =
        broker.get_pending_orders().await.unwrap().pop()
    else {
        panic!("expected a pending order")
    };

    let Order::OCA(oca) = pending_order.order else {
        panic!("expected a once cancels others (OCA) order")
    };
}

#[tokio::test]
async fn cancel_pending_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
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

    let OrderResult::FilledOrder(filled_order) = broker.place_order(&limit_order).await.unwrap()
    else {
        panic!("must get a filled result")
    };

    let pending_order = PendingOrder {
        order_id: filled_order.order_id.to_owned(),
        order: limit_order.to_owned(),
    };
    broker.cancel(&pending_order).await.unwrap();

    let pending_orders = broker.get_pending_orders().await.unwrap();

    assert!(
        pending_orders.is_empty(),
        "failed to remove canceled pending order"
    )
}

#[tokio::test]
async fn update_pending_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
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

    let order_result = broker.place_order(&limit_order).await.unwrap();
    let OrderResult::PendingOrder(p) = order_result else {
        panic!("failed to get a pending order when placing a limit order")
    };

    let pending_order = PendingOrder {
        order_id: p.order_id.to_owned(),
        order: Order::Limit(order::Limit::new(
            20,
            setup.price,
            side,
            setup.security.to_owned(),
            order::TimesInForce::GTC,
        )),
    };

    broker.update(&pending_order).await.unwrap();

    let Some(OrderResult::PendingOrder(result)) = broker.get_pending_orders().await.unwrap().pop()
    else {
        panic!("failed get updated pending order")
    };

    assert_eq!(pending_order, result)
}

#[tokio::test]
async fn close_existing_trade_on_low_balance() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned(), stub.to_owned());

    let quantity_1 = 100;
    let market_order_1 = Order::Market(Market::new(
        quantity_1,
        Side::Long,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_1).await.unwrap();
    let quantity_2 = 100;
    let market_order_2 = Order::Market(Market::new(
        quantity_2,
        Side::Short,
        setup.security.to_owned(),
    ));
    broker.place_order(&market_order_2).await.unwrap();

    let result = broker.get_positions().await.unwrap();

    assert!(
        result.is_empty(),
        "trade should be closed regardless of low balance"
    )
}
