use rust_decimal::Decimal;

pub struct AlgorithmRiskConfig {
    starting_balance: Decimal,
    max_open_trades: Option<u32>,
    max_portfolio_loss: Option<f64>,
    max_portfolio_risk: Option<f64>,
    max_risk_per_trade: Option<f64>,
    max_pending_orders: Option<u32>,
}

struct AlgorithmRiskConfigBuilder {
    starting_balance: Decimal,
    max_open_trades: Option<u32>,
    max_portfolio_loss: Option<f64>,
    max_portfolio_risk: Option<f64>,
    max_risk_per_trade: Option<f64>,
    max_pending_orders: Option<u32>,
}

impl AlgorithmRiskConfigBuilder {
    pub fn default() -> Self {
        Self {
            starting_balance: Decimal::new(0, 0),
            max_pending_orders: None,
            max_portfolio_risk: None,
            max_risk_per_trade: None,
            max_portfolio_loss: Some(1f64),
            max_open_trades: None,
        }
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

    pub fn with_max_pending_orders(mut self, max_pending_orders: u32) -> Self {
        self.max_pending_orders = Some(max_pending_orders);
        self
    }

    pub fn build(self) -> AlgorithmRiskConfig {
        AlgorithmRiskConfig {
            max_pending_orders: self.max_pending_orders,
            starting_balance: self.starting_balance,
            max_open_trades: self.max_open_trades,
            max_risk_per_trade: self.max_risk_per_trade,
            max_portfolio_loss: self.max_portfolio_loss,
            max_portfolio_risk: self.max_portfolio_risk,
        }
    }
}

impl AlgorithmRiskConfig {
    pub fn builder() -> AlgorithmRiskConfigBuilder {
        AlgorithmRiskConfigBuilder::default()
    }
}
