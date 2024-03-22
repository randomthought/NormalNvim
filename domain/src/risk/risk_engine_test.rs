struct Setup;

impl Setup {
    pub fn new() -> Self {
        todo!()
    }
}

#[cfg(test)]
#[tokio::test]
async fn reject_trade_on_halt() {
    let setup = Setup::new();
    todo!()
}

#[tokio::test]
async fn reject_trade_on_portfolio_risk() {
    todo!()
}

#[tokio::test]
async fn reject_trade_on_max_open_trades() {
    todo!()
}

#[tokio::test]
async fn do_not_trade_on_insufficient_balance() {
    todo!()
}
