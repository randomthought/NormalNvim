use crate::{
    data::QouteProvider,
    models::orders::{common::Side, security_position::SecurityPosition},
    order::{Account, OrderReader},
};
use futures_util::future;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::sync::Arc;

#[derive(Debug)]
pub struct Position {
    security_position: SecurityPosition,
    unlrealized_profit: Decimal,
}

impl Position {
    pub fn new(security_position: SecurityPosition, unlrealized_profit: Decimal) -> Self {
        Self {
            security_position,
            unlrealized_profit,
        }
    }
}

pub struct Portfolio {
    account: Arc<dyn Account + Sync + Send>,
    order_reader: Arc<dyn OrderReader + Sync + Send>,
    qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
}

impl Portfolio {
    pub fn new(
        account: Arc<dyn Account + Sync + Send>,
        order_reader: Arc<dyn OrderReader + Sync + Send>,
        qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
    ) -> Self {
        Self {
            order_reader,
            qoute_provider,
            account,
        }
    }

    pub async fn get_open_positions(&self) -> Result<Vec<Position>, crate::error::Error> {
        let orders = self.order_reader.get_positions().await?;

        let futures: Vec<_> = orders
            .iter()
            .map(|sp| async move {
                let quote = self.qoute_provider.get_quote(&sp.security).await?;

                let init = Decimal::from_u64(0).unwrap();
                let profit = sp.holding_details.iter().fold(init, |acc, next| {
                    let q = Decimal::from_u64(next.quantity).unwrap();
                    let profit = match sp.side {
                        Side::Long => (next.price - quote.ask) * q,
                        Side::Short => (quote.bid - next.price) * q,
                    };
                    profit + acc
                });

                let p = Position::new(sp.clone(), profit);

                Ok(p) as Result<Position, crate::error::Error>
            })
            .collect();

        let positions = future::try_join_all(futures).await?;

        Ok(positions)
    }

    // Total portfolio value if we sold all holdings at current market rates.
    pub async fn unrealized_profit(&self) -> Result<Decimal, crate::error::Error> {
        let result: Decimal = self
            .get_open_positions()
            .await?
            .iter()
            .map(|p| p.unlrealized_profit)
            .sum();

        Ok(result)
    }

    pub async fn account_value(&self) -> Result<Decimal, crate::error::Error> {
        self.account.get_account_balance().await
    }

    pub async fn margin_remaining(&self) -> Result<Decimal, crate::error::Error> {
        self.account.get_buying_power().await
    }
}
