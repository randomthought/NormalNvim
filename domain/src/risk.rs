use crate::models::event::Signal;
use crate::models::order::{Order, StopLimitMarket};
use crate::models::security::Security;
use crate::{models::event::Event, order::OrderManager};
use async_trait::async_trait;
use std::io;

enum TradingState {
    Active, // trading is enabled
    // TODO: add an update order type this might be good for only accepting modified orders and not trading
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

pub enum SignalResult {
    Rejected,
    PlacedOrder(Order),
}

#[derive(Debug, Clone, Copy)]
pub struct RiskEngineConfig {
    pub max_portfolio_risk: f32,
    pub max_risk_per_trade: f32,
    pub max_open_trades: Option<u32>,
    // TODO: nautilus_trader has max_order_submit_rate and max_order_modify_rate. Maybe it's worth having
}

impl RiskEngineConfig {
    pub fn new(max_portfolio_risk: f32, max_risk_per_trade: f32) -> Result<Self, String> {
        if (0.0..=1.0).contains(&max_risk_per_trade) {
            return Err("{dbg!(max_risk_per_trade)} has to be between a value 0 and 1".to_owned());
        }

        if (0.0..=1.0).contains(&max_portfolio_risk) {
            return Err("{dbg!(max_portfolio_risk)} has to be between a value 0 and 1".to_owned());
        }

        if max_portfolio_risk < max_risk_per_trade {
            return Err("risk per trade cannot be greater than portfolio risk".to_owned());
        }

        Ok(Self {
            max_portfolio_risk,
            max_risk_per_trade,
            max_open_trades: None,
        })
    }
}

pub struct RiskEngine {
    risk_engine_config: RiskEngineConfig,
    // TODO: state has to be mutable.
    trading_state: TradingState,
    order_manager: Box<dyn OrderManager + Send + Sync>,
}

impl RiskEngine {
    pub fn new(
        risk_engine_config: RiskEngineConfig,
        order_manager: Box<dyn OrderManager + Send + Sync>,
    ) -> Self {
        Self {
            risk_engine_config,
            trading_state: TradingState::Active,
            order_manager,
        }
    }

    pub async fn process_signal(&self, signal: Signal) -> Result<SignalResult, io::Error> {
        println!("risk_engine processed signal");
        if let TradingState::Halted = self.trading_state {
            return Ok(SignalResult::Rejected);
        }

        let config = &self.risk_engine_config;

        if let Some(max) = config.max_open_trades {
            let open_trades = self.get_open_trades().await?;
            if open_trades >= max {
                return Ok(SignalResult::Rejected);
            }
        }

        let quantity = self.calulate_risk_quantity(&signal.security).await?;
        let order = _to_order(signal, quantity).unwrap();

        self.order_manager.place_order(&order).await?;
        return Ok(SignalResult::PlacedOrder(order));
    }

    async fn get_open_trades(&self) -> Result<u32, io::Error> {
        let results = self.order_manager.orders().await?.len();

        Ok(results as u32)
    }

    async fn calulate_risk_quantity(&self, security: &Security) -> Result<u32, io::Error> {
        todo!()
    }
}

fn _to_order(signal: Signal, quantity: u32) -> Result<Order, String> {
    let stop_limit_market = StopLimitMarket::new(
        signal.security,
        quantity,
        signal.side,
        signal.stop,
        signal.limit,
        signal.times_in_force,
    )?;

    Ok(Order::StopLimitMarket(stop_limit_market))
}
