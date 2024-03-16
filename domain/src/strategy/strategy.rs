use super::{
    algorithm::{Algorithm, StrategyId},
    portfolio::StrategyPortfolio,
};
use crate::{
    event::model::{Market, Signal},
    models::{order::OrderResult, price::Price},
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use eyre::{Ok, OptionExt};
use rust_decimal::{prelude::FromPrimitive, Decimal};

pub struct Strategy {
    algorithm: Box<dyn Algorithm + Send + Sync>,
    portfolio: Box<dyn StrategyPortfolio + Send + Sync>,
    starting_balance: Price,
    max_portfolio_risk: Option<f64>,
    max_risk_per_trade: Option<f64>,
    max_portfolio_loss: Option<f64>,
    max_open_trades: Option<u32>,
}

#[derive(Default)]
pub struct StrategyBuilder {
    algorithm: Option<Box<dyn Algorithm + Send + Sync>>,
    portfolio: Option<Box<dyn StrategyPortfolio + Send + Sync>>,
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
            starting_balance: Decimal::new(0, 0),
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

    pub fn build(self) -> Result<Strategy> {
        let algorithm = self.algorithm.ok_or_eyre("Algorithm is required")?;
        let portfolio = self.portfolio.ok_or_eyre("Portfolio is required")?;

        Ok(Strategy {
            algorithm,
            portfolio,
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

#[async_trait]
impl Algorithm for Strategy {
    fn strategy_id(&self) -> StrategyId {
        self.algorithm.strategy_id()
    }

    async fn on_data(&self, market: &Market) -> Result<Option<Signal>> {
        let Some(signal) = self.algorithm.on_data(market).await? else {
            return Ok(None);
        };

        let strategy_id = self.strategy_id();
        if let Some(max) = self.max_portfolio_loss {
            let profit = self.portfolio.get_profit(strategy_id).await?;
            let max_portfolio_loss = Decimal::from_f64(max).unwrap() * self.starting_balance;
            if profit <= max_portfolio_loss {
                todo!()
            }
        }

        if let Some(max) = self.max_open_trades {
            let open_trades = self.portfolio.get_holdings(strategy_id).await?;
            if open_trades.len() >= max.try_into().unwrap() {
                todo!()
            }
        }

        let acc_balance = self.portfolio.get_profit(strategy_id).await?;
        if acc_balance <= Decimal::new(0, 0) {
            todo!()
        }

        if let Some(mrpt) = self.max_risk_per_trade {
            let max_risk_per_trade = acc_balance * Decimal::from_f64(mrpt).unwrap();
            todo!()
        }

        return Ok(Some(signal));
    }

    async fn on_order(&self, order_result: &OrderResult) -> Result<()> {
        self.algorithm.on_order(order_result).await
    }
}
