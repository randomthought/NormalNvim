use crate::event::EventHandler;
use crate::{models::event::Event, order::OrderManager};
use async_trait::async_trait;
use std::io;

enum TradingState {
    Active, // trading is enabled
    // TODO: add an update order type this might be good for only accepting modified orders and not trading
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

pub struct RiskEngineConfig {
    max_portfolio_risk: f32,
    max_risk_per_trade: f32,
    max_open_trades: Option<i32>,
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

pub struct RiskEngine<'a> {
    risk_engine_config: RiskEngineConfig,
    trading_state: TradingState,
    order_manager: &'a dyn OrderManager,
}

impl<'a> RiskEngine<'a> {
    pub fn new(risk_engine_config: RiskEngineConfig, order_manager: &'a dyn OrderManager) -> Self {
        Self {
            risk_engine_config,
            trading_state: TradingState::Active,
            order_manager,
        }
    }
}

#[async_trait]
impl<'a> EventHandler for RiskEngine<'a> {
    async fn handle(&self, event: &Event) -> Result<(), io::Error> {
        if let Event::Order(_) = event {
            if let TradingState::Halted = self.trading_state {
                // TODO: Are you sure you want to return nothing if trading state is halted?
                return Ok(());
            }
        }

        return Ok(());
    }
}
