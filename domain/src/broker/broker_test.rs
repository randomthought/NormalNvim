use core::panic;
use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::sync::RwLock;

use crate::{
    broker::broker::Broker,
    data::QouteProvider,
    models::{
        orders::{
            common::{Side, TimeInForce},
            market::Market,
            new_order::NewOrder,
            one_cancels_others::OneCancelsOthers,
            order_result::OrderResult,
            pending_order::PendingOrder,
            security_position::{HoldingDetail, SecurityPosition},
            stop_limit_market::StopLimitMarket,
        },
        price::{common::Price, quote::Quote},
        security::{AssetType, Exchange, Security},
    },
    strategy::{algorithm::StrategyId, portfolio::StrategyPortfolio},
};
use crate::{
    models::orders::limit::Limit,
    order::{Account, OrderManager, OrderReader},
};

const strategy_id: StrategyId = "fake_algo";

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
    async fn get_quote(&self, security: &Security) -> Result<Quote, crate::error::Error> {
        let price = self.price.read().await;

        let quote = Quote::builder()
            .with_security(security.to_owned())
            .with_bid(*price)
            .with_ask(*price)
            .with_ask_size(0)
            .with_bid_size(0)
            .with_timestamp(Duration::new(5, 0))
            .build()
            .unwrap();

        Ok(quote)
    }
}

#[tokio::test]
async fn get_account_balance() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let results = broker.get_account_balance().await.unwrap();

    assert_eq!(balance, results)
}

#[tokio::test]
async fn close_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let quantity_1 = 10;
    let market_order_1 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_1).await.unwrap();
    let quantity_2 = 10;
    let market_order_2 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_2)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_2).await.unwrap();
    let market_order_3 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Short)
            .with_quantity(quantity_1 + quantity_2)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_3).await.unwrap();

    let result = broker.get_positions().await.unwrap();

    assert!(result.is_empty(), "the shouldn't be any open positions");
}

#[tokio::test]
async fn flip_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let quantity_1 = 10;
    let market_order_1 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_1).await.unwrap();
    let quantity_2 = 10;
    let market_order_2 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_2)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_2).await.unwrap();
    let quantity_3 = 40;
    let market_order_3 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Short)
            .with_quantity(quantity_3)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_3).await.unwrap();

    let result = broker.get_positions().await.unwrap();

    let expected = vec![SecurityPosition {
        security: setup.security.to_owned(),
        side: Side::Short,
        holding_details: vec![HoldingDetail {
            strategy_id,
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
    let broker = Broker::new(balance, stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let market = Market::builder()
        .with_security(setup.security.to_owned())
        .with_side(side)
        .with_quantity(quantity)
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();
    let market_order = NewOrder::Market(market);
    broker.place_order(&market_order).await.unwrap();

    let result = broker.get_account_balance().await.unwrap();

    let expected = Decimal::new(90_000, 0);

    assert_eq!(expected, result);
}

#[tokio::test]
async fn get_balance_after_profit() {
    let setup = Setup::new();
    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(1000, 0);
    let broker = Broker::new(balance, stub.to_owned());
    let quantity = 1;
    let side = Side::Long;
    let market = Market::builder()
        .with_security(setup.security.to_owned())
        .with_side(side)
        .with_quantity(quantity)
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();
    let market_order = NewOrder::Market(market);
    let _ = broker.place_order(&market_order).await.unwrap();
    stub.add_to_price(Decimal::new(1000, 0)).await;
    let market_order_close = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Short)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    let _ = broker.place_order(&market_order_close).await.unwrap();
    let balance_after_trade = broker.get_account_balance().await.unwrap();

    let expected = Decimal::new(2000, 0);
    assert_eq!(balance_after_trade, expected)
}

#[tokio::test]
async fn get_balance_after_loss() {
    let setup = Setup::new();
    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(1000, 0);
    let broker = Broker::new(balance, stub.to_owned());
    let quantity = 1;
    let side = Side::Long;
    let market = Market::builder()
        .with_security(setup.security.to_owned())
        .with_side(side)
        .with_quantity(quantity)
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();

    let market_order = NewOrder::Market(market);
    let _ = broker.place_order(&market_order).await.unwrap();
    stub.add_to_price(Decimal::new(-1000, 0)).await;
    let market_order_close = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Short)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    let _ = broker.place_order(&market_order_close).await.unwrap();
    let balance_after_trade = broker.get_account_balance().await.unwrap();

    let expected = Decimal::new(0, 0);
    assert_eq!(balance_after_trade, expected)
}

#[tokio::test]
async fn get_positions() {
    let setup = Setup::new();
    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let market = Market::builder()
        .with_security(setup.security.to_owned())
        .with_side(side)
        .with_quantity(quantity)
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();
    let market_order = NewOrder::Market(market);
    broker.place_order(&market_order).await.unwrap();

    let expected = vec![SecurityPosition {
        security: setup.security.to_owned(),
        side,
        holding_details: vec![HoldingDetail {
            strategy_id,
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
    let broker = Broker::new(balance, stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let pending_order = NewOrder::Limit(
        Limit::builder()
            .with_side(side)
            .with_strategy_id(strategy_id)
            .with_quantity(quantity)
            .with_security(setup.security.to_owned())
            .with_times_in_force(TimeInForce::GTC)
            .with_price(setup.price)
            .build()
            .unwrap(),
    );
    broker.place_order(&pending_order).await.unwrap();

    let result = broker.get_pending_orders().await.unwrap();

    assert!(
        result.is_empty() == false,
        "pending order was inserted but none is returned"
    )
}

#[tokio::test]
async fn insert_market_stop_limit_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let quantity = 10;
    let side = Side::Long;
    let limit_price = Decimal::new(100, 0);
    let stop_price = Decimal::new(90, 0);
    let stop_limit_market = StopLimitMarket::builder()
        .with_security(setup.security.to_owned())
        .with_quantity(quantity)
        .with_limit_side(side)
        .with_stop_price(stop_price)
        .with_limit_price(limit_price)
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();
    let order = NewOrder::StopLimitMarket(stop_limit_market);
    let OrderResult::PendingOrder(pending_order) = broker.place_order(&order).await.unwrap() else {
        panic!("expected a pending order")
    };

    let expected_1 = vec![SecurityPosition {
        security: setup.security.to_owned(),
        side,
        holding_details: vec![HoldingDetail {
            strategy_id,
            quantity,
            price: setup.price,
        }],
    }];

    let results_1 = broker.get_positions().await.unwrap();

    assert_eq!(expected_1, results_1);

    let oco = OneCancelsOthers::builder()
        .with_quantity(quantity)
        .with_strategy_id(strategy_id)
        .with_security(setup.security.to_owned())
        .with_time_in_force(TimeInForce::GTC)
        .add_limit(Side::Short, stop_price)
        .add_limit(side, limit_price)
        .build()
        .unwrap();

    let expected_2: Vec<OrderResult> = vec![OrderResult::PendingOrder(PendingOrder {
        order_id: pending_order.order_id.to_owned(),
        order: NewOrder::OCO(oco),
    })];

    let result_2 = broker.get_pending_orders().await.unwrap();

    assert_eq!(expected_2, result_2);
}

#[tokio::test]
async fn cancel_oco_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let quantity = 10;
    let side = Side::Long;
    let limit_price = Decimal::new(100, 0);
    let stop_price = Decimal::new(90, 0);
    let stop_limit_market = StopLimitMarket::builder()
        .with_security(setup.security.to_owned())
        .with_quantity(quantity)
        .with_limit_side(side)
        .with_stop_price(stop_price)
        .with_limit_price(limit_price)
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();

    let order = NewOrder::StopLimitMarket(stop_limit_market);
    let order_result = broker.place_order(&order).await.unwrap();

    let OrderResult::PendingOrder(pending_order) = order_result else {
        panic!("pending order should be returned when placing limit order")
    };

    broker.cancel(&pending_order).await.unwrap();

    let pending_orders = broker.get_pending_orders().await.unwrap();

    assert!(
        pending_orders.is_empty() == true,
        "pending order was noce properly canceled"
    );
}

#[tokio::test]
async fn cancel_market_stop_limit_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let quantity = 10;
    let side = Side::Long;
    let limit_price = Decimal::new(100, 0);
    let stop_price = Decimal::new(90, 0);
    let stop_limit_market = StopLimitMarket::builder()
        .with_security(setup.security.to_owned())
        .with_quantity(quantity)
        .with_limit_side(side)
        .with_stop_price(stop_price)
        .with_limit_price(limit_price)
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();
    let order = NewOrder::StopLimitMarket(stop_limit_market);
    let order_result = broker.place_order(&order).await.unwrap();

    let OrderResult::PendingOrder(pending_order) = order_result else {
        panic!("pending order should be returned when placing limit order")
    };

    broker.cancel(&pending_order).await.unwrap();

    let pending_orders = broker.get_pending_orders().await.unwrap();

    assert!(
        pending_orders.is_empty() == true,
        "pending order was noce properly canceled"
    );
}

#[tokio::test]
async fn cancel_pending_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let limit_order = NewOrder::Limit(
        Limit::builder()
            .with_side(side)
            .with_strategy_id(strategy_id)
            .with_quantity(quantity)
            .with_security(setup.security.to_owned())
            .with_times_in_force(TimeInForce::GTC)
            .with_price(setup.price)
            .build()
            .unwrap(),
    );

    let OrderResult::PendingOrder(po) = broker.place_order(&limit_order).await.unwrap() else {
        panic!("must get a filled result")
    };

    let pending_order = PendingOrder {
        order_id: po.order_id.to_owned(),
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
    let broker = Broker::new(balance, stub.to_owned());
    let quantity = 10;
    let side = Side::Long;
    let limit_order = NewOrder::Limit(
        Limit::builder()
            .with_side(side)
            .with_strategy_id(strategy_id)
            .with_quantity(quantity)
            .with_security(setup.security.to_owned())
            .with_times_in_force(TimeInForce::GTC)
            .with_price(setup.price)
            .build()
            .unwrap(),
    );

    let order_result = broker.place_order(&limit_order).await.unwrap();
    let OrderResult::PendingOrder(p) = order_result else {
        panic!("failed to get a pending order when placing a limit order")
    };

    let pending_order = PendingOrder {
        order_id: p.order_id.to_owned(),
        order: NewOrder::Limit(
            Limit::builder()
                .with_side(side)
                .with_strategy_id(strategy_id)
                .with_quantity(20)
                .with_security(setup.security.to_owned())
                .with_times_in_force(TimeInForce::GTC)
                .with_price(setup.price)
                .build()
                .unwrap(),
        ),
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
    let broker = Broker::new(balance, stub.to_owned());

    let quantity_1 = 100;
    let market_order_1 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    broker.place_order(&market_order_1).await.unwrap();
    let quantity_2 = 100;
    let market_order_2 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Short)
            .with_quantity(quantity_2)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_2).await.unwrap();

    let result = broker.get_positions().await.unwrap();

    assert!(
        result.is_empty(),
        "trade should be closed regardless of low balance"
    )
}

#[tokio::test]
async fn get_algo_holdings() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let quantity = 100;
    let market_order_1 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    broker.place_order(&market_order_1).await.unwrap();
    let results_1 = broker.get_holdings(strategy_id).await.unwrap();
    let expected = vec![SecurityPosition {
        security: setup.security.to_owned(),
        side: Side::Long,
        holding_details: vec![HoldingDetail {
            strategy_id,
            quantity,
            price: setup.price,
        }],
    }];
    assert_eq!(results_1, expected);

    let results_2 = broker.get_holdings("algo_with_no_trades").await.unwrap();
    assert!(
        results_2.is_empty(),
        "the shouldn't be any trade for an aglorthim that didn't trade"
    );
}

#[tokio::test]
async fn get_algo_profits() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Broker::new(balance, stub.to_owned());

    let quantity = 10;
    let market_order_1 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_1).await.unwrap();

    stub.add_to_price(Decimal::new(100, 0)).await;
    let market_order_2 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_2).await.unwrap();

    let market_order_3 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Short)
            .with_quantity(quantity * 2)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );
    broker.place_order(&market_order_3).await.unwrap();

    let result = broker.get_profit(strategy_id).await.unwrap();
    let expected = Decimal::new(1000, 0);

    assert_eq!(result, expected);
}
