use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::order::Account;

use super::Broker;

#[async_trait]
impl Account for Broker {
    async fn get_account_balance(&self) -> Result<Decimal, crate::error::Error> {
        let balance = self.account_balance.read().await;
        Ok(*balance)
    }
    async fn get_buying_power(&self) -> Result<Decimal, crate::error::Error> {
        let balance = self.account_balance.read().await;
        Ok(*balance)
    }
}
