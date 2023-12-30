use std::sync::Arc;

use super::config::RiskEngineConfig;
use crate::data::QouteProvider;
use crate::event::event::EventHandler;
use crate::event::model::{Event, Signal};
use crate::models::order::{self, Order, StopLimitMarket};
use crate::models::price::Quote;
use crate::order::OrderManager;
use crate::portfolio::Portfolio;
use anyhow::{Context, Ok, Result};
use async_trait::async_trait;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub enum SignalResult {
    Rejected(String), // TODO: maybe make Rejected(String) so you can add a reason for rejection
    PlacedOrder(Order),
}

enum TradingState {
    Active, // trading is enabled
    // TODO: add an update order type this might be good for only accepting modified orders and not trading
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

pub struct RiskEngine {
    risk_engine_config: RiskEngineConfig,
    // TODO: state has to be mutable.
    trading_state: TradingState,
    qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
    order_manager: Arc<dyn OrderManager + Send + Sync>,
    portfolio: Box<Portfolio>,
}

impl RiskEngine {
    pub fn new(
        risk_engine_config: RiskEngineConfig,
        qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
        order_manager: Arc<dyn OrderManager + Send + Sync>,
        portfolio: Box<Portfolio>,
    ) -> Self {
        Self {
            risk_engine_config,
            trading_state: TradingState::Active,
            order_manager,
            qoute_provider,
            portfolio,
        }
    }

    pub async fn process_signal(&self, signal: &Signal) -> Result<SignalResult> {
        println!("risk_engine processed signal");
        if let TradingState::Halted = self.trading_state {
            return Ok(SignalResult::Rejected(
                "trading state is in 'halted'".to_owned(),
            ));
        }

        let config = &self.risk_engine_config;

        if let Some(max) = config.max_open_trades {
            let open_trades = self.get_open_trades().await?;
            if open_trades >= max {
                return Ok(SignalResult::Rejected(
                    "exceeded max number of openned".to_owned(),
                ));
            }
        }

        let account_value = self.portfolio.account_value().await?;
        let qoute = self.qoute_provider.get_quote(&signal.security).await?;

        let quantity = self.calulate_risk_quantity(account_value, &qoute, &signal)?;

        if !self.quantity_within_risks_params(quantity, signal.side, &qoute, account_value)? {
            return Ok(SignalResult::Rejected(
                "unable to afford this trade according to portfolio risk params".to_owned(),
            ));
        }

        let order = _to_order(signal, quantity)?;

        self.order_manager.place_order(&order).await?;

        Ok(SignalResult::PlacedOrder(order))
    }

    async fn get_open_trades(&self) -> Result<u32> {
        let results = self.order_manager.open_orders().await?.len();

        Ok(results as u32)
    }

    fn calulate_risk_quantity(
        &self,
        account_value: Decimal,
        qoute: &Quote,
        signal: &Signal,
    ) -> Result<u64> {
        let obtain_price = match signal.side {
            order::Side::Long => qoute.ask,
            order::Side::Short => qoute.bid,
        };

        // TODO: think about making risk engine values all decimals?
        let max_risk_per_trade = Decimal::from_f64(self.risk_engine_config.max_risk_per_trade)
            .context(format!(
                "unable to parse '{}' to decimal",
                self.risk_engine_config.max_risk_per_trade
            ))?;

        let max_trade_loss = account_value * max_risk_per_trade;

        let risk_amount = (obtain_price - signal.stop).abs();

        let quantity = (max_trade_loss / risk_amount).trunc();

        let result = u64::try_from(quantity)?;
        Ok(result)
    }

    fn quantity_within_risks_params(
        &self,
        quantity: u64,
        side: order::Side,
        qoute: &Quote,
        account_value: Decimal,
    ) -> Result<bool> {
        // TODO: unit test needed for this.

        // TODO: think about making risk engine values all decimals?
        let max_trade_portfolio_accumulaton = Decimal::from_f64(
            self.risk_engine_config.max_trade_portfolio_accumulaton,
        )
        .context(format!(
            "unable to convert '{}' to a decimal",
            self.risk_engine_config.max_trade_portfolio_accumulaton
        ))?;

        let max_spend_on_trade = account_value * max_trade_portfolio_accumulaton;

        let obtain_price = match side {
            order::Side::Long => qoute.ask,
            order::Side::Short => qoute.bid,
        };

        let spend_total = obtain_price
            * Decimal::from_u64(quantity)
                .context(format!("unable to convert '{}' to a decimal", quantity))?;

        Ok(spend_total <= max_spend_on_trade)
    }
}

#[async_trait]
impl EventHandler for RiskEngine {
    async fn handle(&self, event: &Event) -> Result<()> {
        if let Event::Signal(s) = event {
            self.process_signal(s).await?;
        }

        Ok(())
    }
}

fn _to_order(signal: &Signal, quantity: u64) -> Result<Order> {
    let stop_limit_market = StopLimitMarket::new(
        signal.security.clone(),
        quantity,
        signal.side,
        signal.stop,
        signal.limit,
        signal.times_in_force,
    )?;

    Ok(Order::StopLimitMarket(stop_limit_market))
}
