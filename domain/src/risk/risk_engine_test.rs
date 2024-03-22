use core::panic;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::sync::RwLock;

use crate::broker::broker::Broker;
use crate::models::orders::common::Side;
use crate::models::orders::market::Market;
use crate::models::orders::new_order::NewOrder;
use crate::risk::algo_risk_config::AlgorithmRiskConfig;
use crate::risk::error::RiskError;
use crate::risk::risk_engine::TradingState;
use crate::strategy::algorithm::StrategyId;
use crate::strategy::model::signal::{Entry, Signal};
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
        .with_strategy_portrfolio(broker.clone())
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
}

#[tokio::test]
async fn two_algos_cannot_trade_same_instrument() {
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
        .add_algorithm_risk_config(algo_risk_config)
        .with_qoute_provider(stub.clone())
        .with_strategy_portrfolio(broker.clone())
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
            .with_strategy_id("strategy_2")
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
        Ok(result) => panic!("second trade cannot be succesful: {:?}", result),
    }
}

#[tokio::test]
async fn trading_state_reduce() {
    todo!()
}

#[tokio::test]
async fn trading_state_reduce_on_modify() {
    todo!()
}

#[tokio::test]
async fn reject_trade_on_portfolio_risk() {
    todo!()
}

#[tokio::test]
async fn reject_trade_on_max_open_trades() {
    todo!()
}

#[tokio::test]
async fn do_not_trade_on_insufficient_balance() {
    todo!()
}
