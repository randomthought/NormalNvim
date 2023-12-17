use crate::{
    data::QouteProvider,
    models::order::{FilledOrder, OrderResult, Side},
    order::{Account, OrderReader},
};
use anyhow::Result;
use futures_util::future;
use rust_decimal::Decimal;
use std::sync::Arc;

#[derive(Debug)]
pub struct Position {
    filled_order: FilledOrder,
    unlrealized_profit: Decimal,
}

impl Position {
    pub fn new(filled_order: FilledOrder, unlrealized_profit: Decimal) -> Self {
        Self {
            filled_order,
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

    pub async fn get_open_positions(&self) -> Result<Vec<Position>> {
        let orders = self.order_reader.orders().await?;

        // let futures: Vec<impl Future<Output = Result<Option<Position>, io::Error>>> = orders
        let futures: Vec<_> = orders
            .iter()
            .flat_map(|order| match order {
                OrderResult::FilledOrder(o) => Some(o),
                _ => None,
            })
            .map(|order| async move {
                let quote = self.qoute_provider.get_quote(&order.security).await?;

                let profit = match order.side {
                    Side::Long => order.price - quote.ask,
                    Side::Short => quote.bid - order.price,
                };

                let p = Position::new(order.clone(), profit);

                Ok(p) as Result<Position>
            })
            .collect();

        let positions = future::try_join_all(futures).await?;

        Ok(positions)
    }

    // Total portfolio value if we sold all holdings at current market rates.
    pub async fn unrealized_profit(&self) -> Result<Decimal> {
        let result: Decimal = self
            .get_open_positions()
            .await?
            .iter()
            .map(|p| p.unlrealized_profit)
            .sum();

        Ok(result)
    }

    pub async fn account_value(&self) -> Result<Decimal> {
        self.account.get_account_balance().await
    }

    pub async fn margin_remaining(&self) -> Result<Decimal> {
        self.account.get_buying_power().await
    }
}
