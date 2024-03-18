use super::{
    algorithm::Algorithm,
    error::SignalError,
    model::{
        algo_event::AlgoEvent,
        signal::{Entry, Signal},
    },
    portfolio::StrategyPortfolio,
};
use crate::{
    data::QouteProvider,
    models::{
        orders::{common::Side, new_order::NewOrder},
        price::Price,
        security::Security,
    },
};
use rust_decimal::{
    prelude::{FromPrimitive, Signed},
    Decimal,
};

pub struct Strategy {
    algorithm: Box<dyn Algorithm + Send + Sync>,
    portfolio: Box<dyn StrategyPortfolio + Send + Sync>,
    qoute_provider: Box<dyn QouteProvider + Send + Sync>,
    starting_balance: Price,
    max_open_trades: Option<u32>,
    max_portfolio_loss: Option<f64>,
    max_portfolio_risk: Option<f64>,
    max_risk_per_trade: Option<f64>,
}

#[derive(Default)]
pub struct StrategyBuilder {
    algorithm: Option<Box<dyn Algorithm + Send + Sync>>,
    portfolio: Option<Box<dyn StrategyPortfolio + Send + Sync>>,
    qoute_provider: Option<Box<dyn QouteProvider + Send + Sync>>,
    starting_balance: Decimal,
    max_portfolio_risk: Option<f64>,
    max_portfolio_loss: Option<f64>,
    max_risk_per_trade: Option<f64>,
    max_open_trades: Option<u32>,
}

impl StrategyBuilder {
    pub fn new() -> Self {
        StrategyBuilder {
            algorithm: None,
            portfolio: None,
            qoute_provider: None,
            starting_balance: Decimal::new(0, 0),
            // TODO: maybe add max pending orders?
            max_portfolio_risk: None,
            max_risk_per_trade: None,
            max_portfolio_loss: Some(1f64),
            max_open_trades: None,
        }
    }
    pub fn with_algorithm(mut self, algo: Box<dyn Algorithm + Send + Sync>) -> Self {
        self.algorithm = Some(algo);
        self
    }

    pub fn with_portfolio(mut self, portfolio: Box<dyn StrategyPortfolio + Send + Sync>) -> Self {
        self.portfolio = Some(portfolio);
        self
    }

    pub fn with_qoute_provider(
        mut self,
        qoute_provider: Box<dyn QouteProvider + Send + Sync>,
    ) -> Self {
        self.qoute_provider = Some(qoute_provider);
        self
    }

    pub fn with_starting_balance(mut self, starting_balance: Decimal) -> Self {
        self.starting_balance = starting_balance;
        self
    }

    pub fn with_max_portfolio_risk(mut self, max_portfolio_risk: f64) -> Self {
        self.max_portfolio_risk = Some(max_portfolio_risk);
        self
    }

    pub fn with_max_portfolio_loss(mut self, max_portfolio_loss: f64) -> Self {
        self.max_portfolio_loss = Some(max_portfolio_loss);
        self
    }

    pub fn with_max_risk_per_trade(mut self, max_risk_per_trade: f64) -> Self {
        self.max_risk_per_trade = Some(max_risk_per_trade);
        self
    }

    pub fn with_open_trades(mut self, max_open_trades: u32) -> Self {
        self.max_open_trades = Some(max_open_trades);
        self
    }

    pub fn build(self) -> Result<Strategy, String> {
        let algorithm = self.algorithm.ok_or("algorithm is required".to_string())?;
        let portfolio = self.portfolio.ok_or("portfolio is required".to_string())?;
        let qoute_provider = self
            .qoute_provider
            .ok_or("qoute_provider is required".to_string())?;

        Ok(Strategy {
            algorithm,
            portfolio,
            qoute_provider,
            starting_balance: self.starting_balance,
            max_open_trades: self.max_open_trades,
            max_risk_per_trade: self.max_risk_per_trade,
            max_portfolio_loss: self.max_portfolio_loss,
            max_portfolio_risk: self.max_portfolio_risk,
        })
    }
}

impl Strategy {
    pub fn builder() -> StrategyBuilder {
        StrategyBuilder::default()
    }
}

impl Strategy {
    pub async fn on_event<'a>(
        &self,
        algo_event: AlgoEvent<'a>,
    ) -> Result<Option<Signal>, SignalError> {
        let Some(signal) = self
            .algorithm
            .on_event(algo_event)
            .await
            .map_err(|e| SignalError::Any(e.into()))?
        else {
            return Ok(None);
        };

        if matches!(signal, Signal::Cancel(_) | Signal::Liquidate(_)) {
            return Ok(Some(signal));
        }

        let mut acc_balance = Decimal::default();

        let strategy_id = self.algorithm.strategy_id();
        if let Some(max) = self.max_portfolio_loss {
            let profit = self
                .portfolio
                .get_profit(strategy_id)
                .await
                .map_err(|e| SignalError::Any(e.into()))?;

            acc_balance = profit + self.starting_balance;

            let max_portfolio_loss = Decimal::from_f64(max).unwrap() * self.starting_balance;
            if profit <= max_portfolio_loss {
                return Err(SignalError::ExceededMaxPortfolioLoss);
            }
        }

        if let Some(max) = self.max_open_trades {
            let open_trades = self
                .portfolio
                .get_holdings(strategy_id)
                .await
                .map_err(|e| SignalError::Any(e.into()))?;

            if open_trades.len() >= max.try_into().unwrap() {
                return Err(SignalError::ExceededMaxOpenTrades);
            }
        }

        // TODO: we should consider the same for pending orders to ensure we are not taking too much risk on an update
        if let (Some(mrpt), Signal::Entry(s)) = (self.max_risk_per_trade, signal.to_owned()) {
            let max_risk_per_trade = acc_balance * Decimal::from_f64(mrpt).unwrap();
            let trade_risk = self.calaulate_trade_risk(&s).await?;
            if trade_risk > max_risk_per_trade {
                return Err(SignalError::SignalExceedsMaxRiskPerTrade(signal));
            }
        }

        return Ok(Some(signal));
    }

    async fn get_market_price(
        &self,
        security: &Security,
        side: Side,
    ) -> Result<Decimal, SignalError> {
        let quote = self
            .qoute_provider
            .get_quote(security)
            .await
            .map_err(|e| SignalError::Any(e.into()))?;

        let price = match side {
            Side::Long => quote.ask,
            Side::Short => quote.bid,
        };

        Ok(price)
    }

    async fn get_trade_cost(&self, entry: &Entry) -> Result<Decimal, SignalError> {
        let order_detailts = entry.order.get_order_details();
        let q = Decimal::from_u64(order_detailts.quantity).unwrap();

        if let NewOrder::Limit(l) = entry.order.to_owned() {
            return Ok(q * l.price);
        }

        let price = self
            .get_market_price(entry.order.get_security(), order_detailts.side)
            .await?;

        let trade_cost = q * price;

        Ok(trade_cost)
    }

    async fn calaulate_trade_risk(&self, entry: &Entry) -> Result<Decimal, SignalError> {
        match entry.order.to_owned() {
            NewOrder::StopLimitMarket(slm) => {
                let order_detailts = slm.market.order_details.to_owned();
                let q = Decimal::from_u64(order_detailts.quantity).unwrap();
                let price = self
                    .get_market_price(entry.order.get_security(), order_detailts.side)
                    .await?;
                let risk = (slm.get_stop().price - price).abs() * q;
                Ok(risk)
            }
            _ => self.get_trade_cost(entry).await,
        }
    }
}
