use crate::models::price::Price;

pub struct Strategy {
    balance: Price,
    algorithms: HashMap<String, Box<dyn Algorithm + Send + Sync>>,
    pub max_portfolio_risk: f64,
    pub max_risk_per_trade: f64,
    pub max_open_trades: Option<u32>,
}

impl Strategy {
    pub fn get_balance(&self) -> Price {
        self.balance.to_owned()
    }
}
