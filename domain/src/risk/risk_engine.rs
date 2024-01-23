use std::sync::Arc;

use super::config::RiskEngineConfig;
use crate::data::QouteProvider;
use crate::event::event::{EventHandler, EventProducer};
use crate::event::model::{AlgoOrder, Event, Signal};
use crate::models::order::{self, NewOrder, Order};
use crate::models::price::Quote;
use crate::order::OrderManager;
use crate::portfolio::Portfolio;
use async_trait::async_trait;
use color_eyre::eyre::Result;
use eyre::ContextCompat;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum SignalResult {
    Rejected(String), // TODO: maybe make Rejected(String) so you can add a reason for rejection
    PlacedOrder(NewOrder),
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
    pub trading_state: TradingState,
    pub qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
    event_producer: Arc<dyn EventProducer + Send + Sync>,
    pub order_manager: Arc<dyn OrderManager + Send + Sync>,
    pub portfolio: Box<Portfolio>,
}

impl RiskEngine {
    pub fn new(
        risk_engine_config: RiskEngineConfig,
        event_producer: Arc<dyn EventProducer + Send + Sync>,
        qoute_provider: Arc<dyn QouteProvider + Send + Sync>,
        order_manager: Arc<dyn OrderManager + Send + Sync>,
        portfolio: Box<Portfolio>,
    ) -> Self {
        Self {
            event_producer,
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

        let config = &self.risk_engine_config;

        if let Some(max) = config.max_open_trades {
            let open_trades = self.get_open_trades().await?;
            if open_trades >= max {
                return Ok(SignalResult::Rejected(format!(
                    "exceeded max number of opened_trades='{open_trades}'"
                )));
            }
        }

        let (security, quantity, side) = match signal.order.clone() {
            NewOrder::Market(o) => (o.security, o.order_details.quantity, o.order_details.side),
            NewOrder::Limit(o) => (o.security, o.order_details.quantity, o.order_details.side),
            NewOrder::StopLimitMarket(o) => (
                o.market.security,
                o.market.order_details.quantity,
                o.market.order_details.side,
            ),
            NewOrder::OCA(_) => todo!(),
        };

        let account_value = self.portfolio.account_value().await?;
        let qoute = self.qoute_provider.get_quote(&security).await?;

        if !self.quantity_within_risks_params(quantity, side, &qoute, account_value)? {
            return Ok(SignalResult::Rejected(
                "unable to afford this trade according to portfolio risk params".to_owned(),
            ));
        }

        let order = signal.order.clone();
        let order_result = self.order_manager.place_order(&order).await?;

        let event = Event::AlgoOrder(AlgoOrder {
            strategy_id: signal.strategy_id.to_owned(),
            order: Order::OrderResult(order_result),
        });

        // TODO: to prevent duplicate events, you might need to only listen for filled limit orders from broker
        self.event_producer.produce(event).await?;

        Ok(SignalResult::PlacedOrder(order))
    }

    async fn get_open_trades(&self) -> Result<u32> {
        let results = self.order_manager.get_positions().await?.len();

        Ok(results as u32)
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
        .wrap_err(format!(
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
                .wrap_err(format!("unable to convert '{}' to a decimal", quantity))?;

        Ok(spend_total <= max_spend_on_trade)
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
