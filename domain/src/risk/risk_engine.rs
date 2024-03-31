use derive_builder::Builder;
use futures_util::{future, StreamExt};
use rust_decimal::prelude::Signed;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::collections::HashMap;
use std::sync::Arc;

use super::algo_risk_config::AlgorithmRiskConfig;
use super::error::RiskError;
use crate::data::QouteProvider;
use crate::models::orders::common::Side;
use crate::models::orders::market::Market;
use crate::models::orders::new_order::NewOrder;
use crate::models::orders::order_result::OrderResult;
use crate::models::orders::pending_order::PendingOrder;
use crate::models::orders::security_position::SecurityPosition;
use crate::models::security::Security;
use crate::order::OrderManager;
use crate::strategy::algorithm::{Strategy, StrategyId};
use crate::strategy::model::signal::{Cancel, Close, Entry, Signal};
use crate::strategy::portfolio::StrategyPortfolio;

#[derive(Clone)]
pub enum TradingState {
    Active,   // trading is enabled
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

#[derive(Builder, Clone)]
#[builder(setter(prefix = "with"))]
pub struct RiskEngine {
    #[builder(default, setter(strip_option))]
    max_portfolio_risk: Option<f64>,
    #[builder(default, setter(strip_option))]
    max_portfolio_risk_per_trade: Option<f64>,
    #[builder(default, setter(strip_option))]
    max_portfolio_open_trades: Option<u32>,
    // TODO: state has to be mutable.
    #[builder(default = "TradingState::Active")]
    trading_state: TradingState,
    #[builder(private)]
    algorithm_risk_configs: HashMap<StrategyId, AlgorithmRiskConfig>,
    qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
    strategy_portfolio: Arc<dyn StrategyPortfolio + Send + Sync>,
    order_manager: Arc<dyn OrderManager + Send + Sync>,
}

impl RiskEngine {
    pub fn builder() -> RiskEngineBuilder {
        RiskEngineBuilder::default()
    }

    pub async fn process_signal(&self, signal: &Signal) -> Result<Vec<OrderResult>, RiskError> {
        // TODO: check the accumulation of orders
        if let TradingState::Halted = self.trading_state {
            return Err(RiskError::TradingHalted);
        }

        let algo_risk_config = self
            .algorithm_risk_configs
            .get(signal.strategy_id())
            .ok_or(RiskError::UnableToFindAlgoRiskConfig(signal.strategy_id()))?;

        let order_results = match signal {
            // TODO: ensure modification doesn't take up more risks
            Signal::Modify(s) => {
                let order_result = self
                    .order_manager
                    .update(s.pending_order())
                    .await
                    .map_err(|e| RiskError::OtherError(e.into()))?;
                Some(vec![order_result])
            }
            Signal::Cancel(s) => {
                let order_result = self
                    .order_manager
                    .cancel(s.order_id())
                    .await
                    .map_err(|e| RiskError::OtherError(e.into()))?;
                Some(vec![order_result])
            }
            Signal::Close(s) => {
                let results = self.close(s).await?;
                Some(results)
            }
            Signal::Liquidate(_) => {
                let order_results = self
                    .liquidate(signal.strategy_id())
                    .await
                    .map_err(|e| RiskError::OtherError(e.into()))?;
                Some(order_results)
            }
            Signal::Entry(_) => None,
        };

        if let Some(order_results) = order_results {
            return Ok(order_results);
        }

        let strategy_id = algo_risk_config.strategy_id();

        let profit = self
            .strategy_portfolio
            .get_profit(strategy_id)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let open_trades = self
            .strategy_portfolio
            .get_security_positions(strategy_id)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let acc_balance =
            algo_account_balance(profit, algo_risk_config.starting_balance, &open_trades[..]);

        if let Some(max) = algo_risk_config.max_portfolio_loss {
            let max_portfolio_loss =
                Decimal::from_f64(max).unwrap() * algo_risk_config.starting_balance;
            if profit <= -max_portfolio_loss {
                return Err(RiskError::ExceededAlgoMaxLoss);
            }
        }

        let open_trades = match (
            algo_risk_config.max_open_trades,
            algo_risk_config.max_risk_per_trade,
            self.max_portfolio_open_trades,
        ) {
            (None, None, None) => vec![],
            _ => self
                .strategy_portfolio
                .get_security_positions(signal.strategy_id())
                .await
                .map_err(|e| RiskError::OtherError(e.into()))?,
        };

        if let Some(max) = self.max_portfolio_open_trades {
            if open_trades.len() >= max as usize {
                return Err(RiskError::ExceededPortfolioOpenTrades);
            }
        }

        if let Some(max) = algo_risk_config.max_open_trades {
            if open_trades.len() >= max as usize {
                return Err(RiskError::ExceededAlgoOpenTrades);
            }
        }

        let Signal::Entry(entry) = signal else {
            return Err(RiskError::UnsupportedSignalType(signal.clone()));
        };

        let trade_risk = match (
            self.max_portfolio_risk_per_trade,
            algo_risk_config.max_risk_per_trade,
        ) {
            (None, None) => Decimal::default(),
            _ => self.get_trade_risk(&entry, &open_trades[..]).await?,
        };

        if let Some(max) = algo_risk_config.max_risk_per_trade {
            let balance = profit + algo_risk_config.starting_balance;
            let max_risk_per_trade = balance * Decimal::from_f64(max).unwrap();
            if trade_risk > max_risk_per_trade {
                return Err(RiskError::ExceededAlgoRiskPerTrade(signal.to_owned()));
            }
        }

        if let Some(max) = self.max_portfolio_risk_per_trade {
            let balance = profit + algo_risk_config.starting_balance;
            let max_risk_per_trade = balance * Decimal::from_f64(max).unwrap();
            if trade_risk > max_risk_per_trade {
                return Err(RiskError::ExceededPortfolioRiskPerTrade);
            }
        }

        let trade_cost = self.get_trade_cost(&entry).await?;
        if trade_cost > acc_balance {
            return Err(RiskError::InsufficientAlgoAccountBalance);
        }

        let all_open_trades = self
            .order_manager
            .get_positions()
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let security_already_traded = all_open_trades
            .iter()
            .filter(|s| {
                s.holding_details
                    .iter()
                    .all(|hd| hd.strategy_id != strategy_id)
            })
            .filter(|s| &s.security == entry.order().get_security())
            .flat_map(|s| s.holding_details.clone());

        if let Some(s) = security_already_traded.last() {
            return Err(RiskError::InstrumentTradedByAglorithm(s.strategy_id));
        }

        let order_result = self
            .order_manager
            .place_order(&entry.order())
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let order_results = vec![order_result];

        Ok(order_results)
    }

    async fn liquidate(&self, strategy_id: StrategyId) -> Result<Vec<OrderResult>, RiskError> {
        let positions = self
            .strategy_portfolio
            .get_security_positions(strategy_id)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let orders_: Result<Vec<NewOrder>, _> = positions
            .iter()
            .map(|sp| {
                let side = match sp.side {
                    Side::Long => Side::Short,
                    Side::Short => Side::Long,
                };
                Market::builder()
                    .with_security(sp.security.to_owned())
                    .with_side(side)
                    .with_quantity(sp.get_quantity())
                    .with_strategy_id(strategy_id)
                    .build()
                    .map(|o| NewOrder::Market(o))
            })
            .collect();

        let orders = orders_.map_err(|e| RiskError::OtherError(e.into()))?;

        let f1 = orders.iter().map(|o| self.order_manager.place_order(&o));

        let pending_orders = self
            .strategy_portfolio
            .get_pending(strategy_id)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let f2 = pending_orders
            .iter()
            .map(|p| self.order_manager.cancel(&p.order_id))
            .chain(f1);

        let order_results = future::try_join_all(f2)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        Ok(order_results)
    }

    async fn get_trade_risk(
        &self,
        entry: &Entry,
        open_trades: &[SecurityPosition],
    ) -> Result<Decimal, RiskError> {
        let strategy_id = entry.order().startegy_id();

        let current = open_trades
            .iter()
            .filter(|v| {
                &v.security == entry.order().get_security()
                    && entry.order().get_order_details().strategy_id().to_owned() == strategy_id
            })
            .last();

        let current_position_risk = match current {
            None => Decimal::default(),
            Some(v) => {
                let pending_orders = self
                    .order_manager
                    .get_pending_orders()
                    .await
                    .map_err(|e| RiskError::OtherError(e.into()))?;

                self.calculate_position_risk(v, &pending_orders[..]).await?
            }
        };

        let signal_risk = self.calaulate_trade_risk(&entry).await?;
        let trade_risk = current_position_risk + signal_risk;

        Ok(trade_risk)
    }

    async fn close(&self, close: &Close) -> Result<Vec<OrderResult>, RiskError> {
        let strategy_id = close.strategy_id();

        let positions = self
            .strategy_portfolio
            .get_security_positions(strategy_id)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let close_orders: Result<Vec<NewOrder>, _> = positions
            .iter()
            .filter(|v| &v.security == close.security())
            .map(|sp| {
                let side = match sp.side {
                    Side::Long => Side::Short,
                    Side::Short => Side::Long,
                };
                Market::builder()
                    .with_security(sp.security.to_owned())
                    .with_side(side)
                    .with_quantity(sp.get_quantity())
                    .with_strategy_id(strategy_id)
                    .build()
                    .map(|o| NewOrder::Market(o))
            })
            .collect();

        let close_orders = close_orders.map_err(|e| RiskError::OtherError(e.into()))?;

        let pending_orders: Vec<_> = self
            .strategy_portfolio
            .get_pending(strategy_id)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let f1 = pending_orders
            .iter()
            .filter(|v| &v.startegy_id() == close.strategy_id())
            .map(|p| self.order_manager.cancel(&p.order_id));

        let f2 = close_orders
            .iter()
            .map(|o| self.order_manager.place_order(&o))
            .chain(f1);

        let order_results = future::try_join_all(f2)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        Ok(order_results)
    }

    async fn get_market_price(
        &self,
        security: &Security,
        side: Side,
    ) -> Result<Decimal, RiskError> {
        let quote = self
            .qoute_provider
            .get_quote(security)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let price = match side {
            Side::Long => quote.ask,
            Side::Short => quote.bid,
        };

        Ok(price)
    }

    async fn get_trade_cost(&self, entry: &Entry) -> Result<Decimal, RiskError> {
        let order_detailts = entry.order().get_order_details();
        let q = Decimal::from_u64(order_detailts.quantity).unwrap();

        if let NewOrder::Limit(l) = entry.order().to_owned() {
            return Ok(q * l.price);
        }

        let price = self
            .get_market_price(entry.order().get_security(), order_detailts.side)
            .await?;

        let trade_cost = q * price;

        Ok(trade_cost)
    }

    async fn calaulate_trade_risk(&self, entry: &Entry) -> Result<Decimal, RiskError> {
        match entry.order().to_owned() {
            NewOrder::StopLimitMarket(slm) => {
                let order_detailts = slm.market.order_details.to_owned();
                let q = Decimal::from_u64(order_detailts.quantity).unwrap();
                let price = self
                    .get_market_price(entry.order().get_security(), order_detailts.side)
                    .await?;
                let risk = (slm.get_stop().price - price).abs() * q;
                Ok(risk)
            }
            _ => self.get_trade_cost(entry).await,
        }
    }

    async fn calculate_position_risk(
        &self,
        security_position: &SecurityPosition,
        pending_orders: &[PendingOrder],
    ) -> Result<Decimal, RiskError> {
        let pending = pending_orders
            .iter()
            .flat_map(|p| match p.order.clone() {
                NewOrder::Market(_) => vec![],
                NewOrder::Limit(v) => vec![v],
                NewOrder::StopLimitMarket(v) => vec![v.get_stop().clone()],
                NewOrder::OCO(v) => v.orders,
            })
            .filter(|p| {
                &p.security == &security_position.security
                    && p.order_details.side != security_position.side
                    && p.order_details.quantity == security_position.get_quantity()
            })
            .last();

        let Some(stop_limit) = pending else {
            let risk = security_position.get_cost();
            return Ok(risk);
        };

        let wap = security_position.get_wieghted_average_price();
        let risk = (stop_limit.price - wap)
            * Decimal::from_u64(stop_limit.order_details.quantity).unwrap();

        Ok(risk)
    }
}

fn algo_account_balance(
    profit: Decimal,
    starting_balance: Decimal,
    open_trades: &[SecurityPosition],
) -> Decimal {
    let open_trades_cost = open_trades
        .iter()
        .fold(Decimal::default(), |acc, n| acc + n.get_cost());

    (profit + starting_balance) - open_trades_cost
}

impl RiskEngineBuilder {
    pub fn add_algorithm_risk_config(
        &mut self,
        algo_risk_config: AlgorithmRiskConfig,
    ) -> &mut Self {
        let strategy_id = algo_risk_config.strategy_id();

        if let Some(config_map) = self.algorithm_risk_configs.as_mut() {
            config_map.insert(strategy_id, algo_risk_config);
            return self;
        };

        let mut map = HashMap::new();
        map.insert(strategy_id, algo_risk_config);
        self.algorithm_risk_configs = Some(map);

        self
    }
}
