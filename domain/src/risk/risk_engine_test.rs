use core::panic;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::sync::RwLock;

use crate::broker::broker::Broker;
use crate::models::orders::common::{Side, TimeInForce};
use crate::models::orders::limit::Limit;
use crate::models::orders::market::Market;
use crate::models::orders::new_order::NewOrder;
use crate::models::orders::pending_order::PendingOrder;
use crate::order::OrderManager;
use crate::risk::algo_risk_config::AlgorithmRiskConfig;
use crate::risk::error::RiskError;
use crate::risk::risk_engine::TradingState;
use crate::strategy::algorithm::StrategyId;
use crate::strategy::model::signal::{Cancel, Entry, Modify, Signal};
use crate::{
    data::QouteProvider,
    models::{
        price::{common::Price, quote::Quote},
        security::{AssetType, Exchange, Security},
    },
};

use super::risk_engine::RiskEngine;

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

#[cfg(test)]
#[tokio::test]
async fn reject_trade_on_halt() {
    use crate::{
        models::orders::pending_order::PendingOrder,
        strategy::model::signal::{Cancel, Modify},
    };

    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .build()
        .unwrap();
    let risk_engine = RiskEngine::builder()
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .with_trading_state(TradingState::Halted)
        .build()
        .unwrap();

    let signal_liquidate = Signal::Liquidate(strategy_id);
    match risk_engine.process_signal(&signal_liquidate).await {
        Err(RiskError::TradingHalted) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("second trade cannot be succesful: {:?}", result),
    }

    let market_order = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order.clone())
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    match risk_engine.process_signal(&entry_signal).await {
        Err(RiskError::TradingHalted) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("second trade cannot be succesful: {:?}", result),
    }

    let modify_order = Modify {
        pending_order: PendingOrder {
            order_id: "pending_order".into(),
            order: market_order.clone(),
        },
        datetime: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    };

    let modify_signal = Signal::Modify(modify_order);
    match risk_engine.process_signal(&modify_signal).await {
        Err(RiskError::TradingHalted) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("second trade cannot be succesful: {:?}", result),
    }

    let cancel_order = Cancel {
        strategy_id,
        order_id: "cancel_order".into(),
        datetime: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    };

    let cancel_signal = Signal::Cancel(cancel_order);
    match risk_engine.process_signal(&cancel_signal).await {
        Err(RiskError::TradingHalted) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("second trade cannot be succesful: {:?}", result),
    }
}

#[tokio::test]
async fn two_algos_cannot_trade_same_instrument() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config_1 = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(balance)
        .build()
        .unwrap();

    let strategy_id_2 = "second_algo";
    let algo_risk_config_2 = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id_2)
        .with_starting_balance(balance)
        .build()
        .unwrap();

    let risk_engine = RiskEngine::builder()
        .add_algorithm_risk_config(algo_risk_config_1)
        .add_algorithm_risk_config(algo_risk_config_2)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

    let quantity = 1;
    let market_order_1 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    let entry_signal_1 = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order_1)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    if let Err(e) = risk_engine.process_signal(&entry_signal_1).await {
        panic!("initial trade not succesful: {:?}", e);
    }

    let market_order_2 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id_2)
            .build()
            .unwrap(),
    );

    let entry_signal_2 = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order_2)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    match risk_engine.process_signal(&entry_signal_2).await {
        Err(RiskError::InstrumentTradedByAglorithm(id)) => {
            if id != strategy_id {
                panic!("incorrent strategy_id=`{:?}` for first trade", id);
            }
        }
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(_) => panic!("second trade cannot be succesful"),
    }
}

#[tokio::test]
async fn reject_trade_on_max_open_trades_zero() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(balance)
        .with_max_open_trades(0)
        .build()
        .unwrap();

    let risk_engine = RiskEngine::builder()
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

    let market_order = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    match risk_engine.process_signal(&entry_signal).await {
        Err(RiskError::ExceededAlgoMaxOpenTrades) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("trade cannot be succesful: {:?}", result),
    }
}

#[tokio::test]
async fn reject_trade_on_max_open_trades() {
    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(balance)
        .with_max_open_trades(4)
        .build()
        .unwrap();

    let risk_engine = RiskEngine::builder()
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

    let signals = ["a", "b", "c", "d", "e"].iter().map(|x| {
        let sec = Security::builder()
            .with_ticker(x.to_string())
            .with_exchange(Exchange::Unknown)
            .with_asset_type(AssetType::Equity)
            .build()
            .unwrap();

        let order = Market::builder()
            .with_security(sec)
            .with_side(Side::Long)
            .with_quantity(1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap();

        Signal::Entry(
            Entry::builder()
                .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
                .with_order(NewOrder::Market(order))
                .with_strength(0.1)
                .build()
                .unwrap(),
        )
    });

    for (i, s) in signals.enumerate() {
        let trade_num = i + 1;
        if trade_num < 5 {
            if let Err(e) = risk_engine.process_signal(&s).await {
                panic!("trade `{}` failed with error: {:?}", trade_num, e);
            }
            continue;
        }

        match risk_engine.process_signal(&s).await {
            Err(RiskError::ExceededAlgoMaxOpenTrades) => (),
            Err(e) => panic!("failed with incorrect error: {:?}", e),
            Ok(result) => panic!("trade cannot be succesful: {:?}", result),
        }
    }
}

#[tokio::test]
async fn do_not_trade_on_insufficient_balance_zero() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(Decimal::default())
        .build()
        .unwrap();

    let risk_engine = RiskEngine::builder()
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

    let market_order = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    match risk_engine.process_signal(&entry_signal).await {
        Err(RiskError::InsufficientAlgoAccountBalance) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("trade cannot be succesful: {:?}", result),
    }
}

#[tokio::test]
async fn do_not_trade_on_insufficient_balance() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(Decimal::new(1_500, 0))
        .build()
        .unwrap();

    let risk_engine = RiskEngine::builder()
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

    let market_order = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(1)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    if let Err(e) = risk_engine.process_signal(&entry_signal).await {
        panic!(
            "failed to make the fist trade when sufficient balance exists: `{:?}`",
            e
        );
    }

    match risk_engine.process_signal(&entry_signal).await {
        Err(RiskError::InsufficientAlgoAccountBalance) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("trade cannot be succesful: {:?}", result),
    }
}

#[tokio::test]
async fn do_not_trade_without_algo_risk_config() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(balance)
        .with_max_open_trades(0)
        .build()
        .unwrap();

    let risk_engine = RiskEngine::builder()
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

    let market_order = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(1)
            .with_strategy_id("some_algo")
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    match risk_engine.process_signal(&entry_signal).await {
        Err(RiskError::UnableToFindAlgoRiskConfig(_)) => (),
        Err(e) => panic!("failed with incorrect error: {:?}", e),
        Ok(result) => panic!("trade cannot be succesful: {:?}", result),
    }
}

#[tokio::test]
async fn trading_state_reduce_market_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(balance)
        .build()
        .unwrap();
    let risk_engine = RiskEngine::builder()
        .with_trading_state(TradingState::Reducing)
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

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

    let market_order_2 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order_2)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    if let Ok(_) = risk_engine.process_signal(&entry_signal).await {
        panic!("cannot add to existing trade on reducing");
    }

    let market_order_3 = NewOrder::Market(
        Market::builder()
            .with_security(setup.security.to_owned())
            .with_side(Side::Short)
            .with_quantity(quantity)
            .with_strategy_id(strategy_id)
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Entry(
        Entry::builder()
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .with_order(market_order_3)
            .with_strength(0.1)
            .build()
            .unwrap(),
    );

    if let Err(e) = risk_engine.process_signal(&entry_signal).await {
        panic!("closing trade not succesful: {:?}", e);
    }
}

#[tokio::test]
async fn trading_state_reduce_on_limit_order() {
    let setup = Setup::new();

    let stub = Arc::new(Stub::new());
    let balance = Decimal::new(100_000, 0);
    let broker = Arc::new(Broker::new(balance, stub.to_owned()));
    let algo_risk_config = AlgorithmRiskConfig::builder()
        .with_strategy_id(strategy_id)
        .with_starting_balance(balance)
        .build()
        .unwrap();
    let risk_engine = RiskEngine::builder()
        .with_trading_state(TradingState::Reducing)
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portfolio(broker.clone())
        .with_order_manager(broker.clone())
        .build()
        .unwrap();

    let quantity_1 = 10;
    let limit_order_1 = NewOrder::Limit(
        Limit::builder()
            .with_security(setup.security.to_owned())
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_1)
            .with_strategy_id(strategy_id)
            .with_times_in_force(TimeInForce::GTC)
            .with_price(Decimal::new(100, 0))
            .build()
            .unwrap(),
    );

    let order_result = broker.place_order(&limit_order_1).await.unwrap();

    let quantity_2 = 20;
    let limit_order_2 = NewOrder::Limit(
        Limit::builder()
            .with_security(setup.security.to_owned())
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_2)
            .with_strategy_id(strategy_id)
            .with_times_in_force(TimeInForce::GTC)
            .with_price(Decimal::new(100, 0))
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Modify(
        Modify::builder()
            .with_pending_order(
                PendingOrder::builder()
                    .with_order(limit_order_2)
                    .with_order_id(order_result.order_id().to_owned())
                    .build()
                    .unwrap(),
            )
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .build()
            .unwrap(),
    );

    if let Ok(_) = risk_engine.process_signal(&entry_signal).await {
        panic!("cannot add more quantity to existing pending order when trading state is reducing");
    }

    let quantity_3 = 5;
    let limit_order_3 = NewOrder::Limit(
        Limit::builder()
            .with_security(setup.security.to_owned())
            .with_security(setup.security.to_owned())
            .with_side(Side::Long)
            .with_quantity(quantity_3)
            .with_strategy_id(strategy_id)
            .with_times_in_force(TimeInForce::GTC)
            .with_price(Decimal::new(100, 0))
            .build()
            .unwrap(),
    );

    let entry_signal = Signal::Modify(
        Modify::builder()
            .with_pending_order(
                PendingOrder::builder()
                    .with_order(limit_order_3)
                    .with_order_id(order_result.order_id().to_owned())
                    .build()
                    .unwrap(),
            )
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .build()
            .unwrap(),
    );

    if let Err(_) = risk_engine.process_signal(&entry_signal).await {
        panic!("unable to reducing quantity for pending order");
    }

    let cancel_signal = Signal::Cancel(
        Cancel::builder()
            .with_order_id(order_result.order_id().to_owned())
            .with_strategy_id(strategy_id)
            .with_datetime(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .build()
            .unwrap(),
    );

    if let Err(_) = risk_engine.process_signal(&cancel_signal).await {
        panic!("unable to reducing pending order by canceling it");
    }
}

#[tokio::test]
async fn reject_trade_on_portfolio_risk_market() {
    todo!()
}

#[tokio::test]
async fn reject_trade_on_portfolio_risk_limit() {
    todo!()
}
