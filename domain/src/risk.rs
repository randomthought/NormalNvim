use crate::data::QouteProvider;
use crate::models::event::Signal;
use crate::models::order::{self, Order, StopLimitMarket};
use crate::models::price::Quote;
use crate::models::security::Security;
use crate::order::OrderManager;
use crate::portfolio::Portfolio;
use std::io;

enum TradingState {
    Active, // trading is enabled
    // TODO: add an update order type this might be good for only accepting modified orders and not trading
    Reducing, // only new orders or updates which reduce an open position are allowed
    Halted,   // all trading commands except cancels are denied
}

pub enum SignalResult {
    Rejected(String), // TODO: maybe make Rejected(String) so you can add a reason for rejection
    PlacedOrder(Order),
}

#[derive(Debug, Clone, Copy)]
pub struct RiskEngineConfig {
    pub max_trade_portfolio_accumulaton: f64,
    // TODO: add functionality to check portfolio risk on orders
    pub max_portfolio_risk: f64,
    pub max_risk_per_trade: f64,
    pub max_open_trades: Option<u32>,
    // TODO: nautilus_trader has max_order_submit_rate and max_order_modify_rate. Maybe it's worth having
}

impl RiskEngineConfig {
    pub fn new(
        max_portfolio_risk: f64,
        max_risk_per_trade: f64,
        max_trade_portfolio_accumulaton: f64,
    ) -> Result<Self, String> {
        if (0.0..=1.0).contains(&max_risk_per_trade) {
            return Err("{dbg!(max_risk_per_trade)} has to be between a value 0 and 1".to_owned());
        }

        if (0.0..=1.0).contains(&max_portfolio_risk) {
            return Err("{dbg!(max_portfolio_risk)} has to be between a value 0 and 1".to_owned());
        }

        if (0.0..=1.0).contains(&max_trade_portfolio_accumulaton) {
            return Err(
                "{dbg!(max_trade_portfolio_accumulaton)} has to be between a value 0 and 1"
                    .to_owned(),
            );
        }

        if max_portfolio_risk < max_risk_per_trade {
            return Err("risk per trade cannot be greater than portfolio risk".to_owned());
        }

        Ok(Self {
            max_portfolio_risk,
            max_risk_per_trade,
            max_open_trades: None,
            max_trade_portfolio_accumulaton,
        })
    }
}

pub struct RiskEngine {
    risk_engine_config: RiskEngineConfig,
    // TODO: state has to be mutable.
    trading_state: TradingState,
    qoute_provider: Box<dyn QouteProvider + Send + Sync>,
    order_manager: Box<dyn OrderManager + Send + Sync>,
    portfolio: Box<Portfolio>,
}

impl RiskEngine {
    pub fn new(
        risk_engine_config: RiskEngineConfig,
        order_manager: Box<dyn OrderManager + Send + Sync>,
        qoute_provider: Box<dyn QouteProvider + Send + Sync>,
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

    pub async fn process_signal(&self, signal: Signal) -> Result<SignalResult, io::Error> {
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

        let quantity = self.calulate_risk_quantity(account_value, &qoute, &signal);

        if !self.quantity_within_risks_params(quantity, signal.side, &qoute, account_value) {
            return Ok(SignalResult::Rejected(
                "unable to afford this trade according to portfolio risk params".to_owned(),
            ));
        }

        let order = _to_order(signal, quantity).unwrap();

        self.order_manager.place_order(&order).await?;

        Ok(SignalResult::PlacedOrder(order))
    }

    async fn get_open_trades(&self) -> Result<u32, io::Error> {
        let results = self.order_manager.orders().await?.len();

        Ok(results as u32)
    }

    fn calulate_risk_quantity(&self, account_value: f64, qoute: &Quote, signal: &Signal) -> u64 {
        let obtain_price = match signal.side {
            order::Side::Long => qoute.ask,
            order::Side::Short => qoute.bid,
        };

        let max_trade_loss = account_value * self.risk_engine_config.max_risk_per_trade;

        let risk_amount = f64::abs(obtain_price - signal.stop);

        let quantity = (max_trade_loss / risk_amount).floor();

        quantity as u64
    }

    fn quantity_within_risks_params(
        &self,
        quantity: u64,
        side: order::Side,
        qoute: &Quote,
        account_value: f64,
    ) -> bool {
        // TODO: unit test needed for this.
        let max_spend_on_trade =
            self.risk_engine_config.max_trade_portfolio_accumulaton * account_value;

        let obtain_price = match side {
            order::Side::Long => qoute.ask,
            order::Side::Short => qoute.bid,
        };

        let spend_total = obtain_price * (quantity as f64);

        spend_total <= max_spend_on_trade
    }
}

fn _to_order(signal: Signal, quantity: u64) -> Result<Order, String> {
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
