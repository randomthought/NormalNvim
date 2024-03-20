use futures_util::future;
use std::sync::Arc;

use super::config::RiskEngineConfig;
use super::error::RiskError;
use crate::data::QouteProvider;
use crate::models::orders::common::Side;
use crate::models::orders::market::Market;
use crate::models::orders::new_order::NewOrder;
use crate::models::orders::order_result::OrderResult;
use crate::order::OrderManager;
use crate::portfolio::Portfolio;
use crate::strategy::algorithm::StrategyId;
use crate::strategy::model::signal::Signal;
use crate::strategy::portfolio::StrategyPortfolio;

#[derive(Debug)]
pub enum SignalResult {
    Rejected(String), // TODO: maybe make Rejected(String) so you can add a reason for rejection
    PlacedOrder(Vec<OrderResult>),
}

#[derive(Clone)]
enum TradingState {
    Active, // trading is enabled
    // TODO: add an update order type this might be good for only accepting modified orders and not trading
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

#[derive(Clone)]
pub struct RiskEngine {
    pub risk_engine_config: RiskEngineConfig,
    // TODO: state has to be mutable.
    trading_state: TradingState,
    pub qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
    strategy_portrfolio: Arc<dyn StrategyPortfolio + Send + Sync>,
    pub order_manager: Arc<dyn OrderManager + Send + Sync>,
    pub portfolio: Box<Portfolio>,
}

impl RiskEngine {
    pub fn new(
        risk_engine_config: RiskEngineConfig,
        strategy_portrfolio: Arc<dyn StrategyPortfolio + Send + Sync>,
        qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
        order_manager: Arc<dyn OrderManager + Send + Sync>,
        portfolio: Box<Portfolio>,
    ) -> Self {
        Self {
            strategy_portrfolio,
            risk_engine_config,
            trading_state: TradingState::Active,
            order_manager,
            qoute_provider,
            portfolio,
        }
    }

    pub async fn process_signal(&self, signal: &Signal) -> Result<Vec<OrderResult>, RiskError> {
        // TODO: check the accumulation of orders
        if let TradingState::Halted = self.trading_state {
            return Err(RiskError::TradingHalted);
        }

        let order_results = match signal {
            Signal::Modify(s) => {
                let order_result = self
                    .order_manager
                    .update(&s.pending_order)
                    .await
                    .map_err(|e| RiskError::OtherError(e.into()))?;
                Some(vec![order_result])
            }
            Signal::Cancel(s) => {
                let order_result = self
                    .order_manager
                    .cancel(&s.pending_order)
                    .await
                    .map_err(|e| RiskError::OtherError(e.into()))?;
                Some(vec![order_result])
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

        let config = &self.risk_engine_config;

        if let Some(max) = config.max_open_trades {
            let open_trades = self
                .get_open_trades()
                .await
                .map_err(|e| RiskError::OtherError(e.into()))?;

            if open_trades >= max {
                return Err(RiskError::ExceededMaxOpenPortfolioTrades);
            }
        }

        let Signal::Entry(s) = signal else {
            return Err(RiskError::UnsupportedSignalType);
        };

        let order_result = self
            .order_manager
            .place_order(&s.order)
            .await
            .map_err(|e| RiskError::OtherError(e.into()))?;

        let order_results = vec![order_result];

        Ok(order_results)
    }

    async fn liquidate(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<OrderResult>, crate::error::Error> {
        let positions = self.strategy_portrfolio.get_holdings(strategy_id).await?;

        let orders: Vec<NewOrder> = positions
            .iter()
            .map(|sp| {
                let side = match sp.side {
                    Side::Long => Side::Short,
                    Side::Short => Side::Long,
                };
                let order =
                    Market::new(sp.get_quantity(), side, sp.security.to_owned(), strategy_id);
                NewOrder::Market(order)
            })
            .collect();

        let f1 = orders.iter().map(|o| self.order_manager.place_order(&o));

        let pending_orders = self.strategy_portrfolio.get_pending(strategy_id).await?;

        let f2 = pending_orders
            .iter()
            .map(|p| self.order_manager.cancel(p))
            .chain(f1);

        let order_results = future::try_join_all(f2).await?;

        Ok(order_results)
    }

    async fn get_open_trades(&self) -> Result<u32, crate::error::Error> {
        let results = self.order_manager.get_positions().await?.len();

        Ok(results as u32)
    }
}
