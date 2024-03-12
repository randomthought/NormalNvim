use futures_util::future;
use std::sync::Arc;

use super::config::RiskEngineConfig;
use crate::data::QouteProvider;
use crate::event::event::{EventHandler, EventProducer};
use crate::event::model::{Event, Signal};
use crate::models::order::{self, Market, NewOrder, Order, OrderResult};
use crate::order::OrderManager;
use crate::portfolio::Portfolio;
use crate::strategy::algorithm::StrategyId;
use crate::strategy::portfolio::StrategyPortfolio;
use async_trait::async_trait;
use color_eyre::eyre::Result;
use eyre::Ok;

#[derive(Debug)]
pub enum SignalResult {
    Rejected(String), // TODO: maybe make Rejected(String) so you can add a reason for rejection
    PlacedOrder(Vec<OrderResult>),
}

enum TradingState {
    Active, // trading is enabled
    // TODO: add an update order type this might be good for only accepting modified orders and not trading
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

pub struct RiskEngine {
    pub risk_engine_config: RiskEngineConfig,
    // TODO: state has to be mutable.
    trading_state: TradingState,
    pub qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
    strategy_portrfolio: Arc<dyn StrategyPortfolio + Send + Sync>,
    event_producer: Arc<dyn EventProducer + Send + Sync>,
    pub order_manager: Arc<dyn OrderManager + Send + Sync>,
    pub portfolio: Box<Portfolio>,
}

impl RiskEngine {
    pub fn new(
        risk_engine_config: RiskEngineConfig,
        strategy_portrfolio: Arc<dyn StrategyPortfolio + Send + Sync>,
        event_producer: Arc<dyn EventProducer + Send + Sync>,
        qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
        order_manager: Arc<dyn OrderManager + Send + Sync>,
        portfolio: Box<Portfolio>,
    ) -> Self {
        Self {
            event_producer,
            strategy_portrfolio,
            risk_engine_config,
            trading_state: TradingState::Active,
            order_manager,
            qoute_provider,
            portfolio,
        }
    }

    pub async fn process_signal(&self, signal: &Signal) -> Result<SignalResult> {
        // TODO: check the accumulation of orders
        if let TradingState::Halted = self.trading_state {
            return Ok(SignalResult::Rejected(
                "trading state is in 'halted'".to_owned(),
            ));
        }

        let order_results = match signal {
            Signal::Modify(s) => {
                let order_result = self.order_manager.update(&s.pending_order).await?;
                Some(vec![order_result])
            }
            Signal::Cancel(s) => {
                let order_result = self.order_manager.cancel(&s.pending_order).await?;
                Some(vec![order_result])
            }
            Signal::Liquidate(_) => {
                let order_results = self.liquidate(signal.strategy_id()).await?;
                Some(order_results)
            }
            Signal::Entry(_) => None,
        };

        if let Some(order_results) = order_results {
            self.report_events(&order_results).await?;
            return Ok(SignalResult::PlacedOrder(order_results));
        }

        let config = &self.risk_engine_config;

        if let Some(max) = config.max_open_trades {
            let open_trades = self.get_open_trades().await?;
            if open_trades >= max {
                return Ok(SignalResult::Rejected(format!(
                    "exceeded max number of opened_trades='{open_trades}'"
                )));
            }
        }

        let Signal::Entry(s) = signal else {
            return Ok(SignalResult::Rejected(format!("Unsupported Signal type")));
        };

        let order_result = self.order_manager.place_order(&s.order).await?;
        let order_results = vec![order_result];
        self.report_events(&order_results).await?;
        return Ok(SignalResult::PlacedOrder(order_results));
    }

    async fn liquidate(&self, strategy_id: StrategyId) -> Result<Vec<OrderResult>> {
        let positions = self.strategy_portrfolio.get_holdings(strategy_id).await?;

        let orders: Vec<NewOrder> = positions
            .iter()
            .map(|sp| {
                let side = match sp.side {
                    order::Side::Long => order::Side::Short,
                    order::Side::Short => order::Side::Long,
                };
                let order =
                    Market::new(sp.get_quantity(), side, sp.security.to_owned(), strategy_id);
                NewOrder::Market(order)
            })
            .collect();

        let f1 = orders.iter().map(|o| self.order_manager.place_order(&o));

        let order_results = future::try_join_all(f1).await?;

        Ok(order_results)
    }

    async fn report_events(&self, order_results: &Vec<OrderResult>) -> Result<()> {
        let f2 = order_results.iter().map(|or| {
            let event = Event::Order(Order::OrderResult(or.to_owned()));
            self.event_producer.produce(event)
        });

        future::try_join_all(f2).await?;

        Ok(())
    }

    async fn get_open_trades(&self) -> Result<u32> {
        let results = self.order_manager.get_positions().await?.len();

        Ok(results as u32)
    }
}

#[async_trait]
impl EventHandler for RiskEngine {
    async fn handle(&self, event: &Event) -> Result<()> {
        if let Event::Signal(s) = event {
            let signal_results = self.process_signal(s).await?;
            println!("signal result: {:?}", signal_results);
        }

        Ok(())
    }
}
