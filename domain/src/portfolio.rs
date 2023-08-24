use std::io;

use futures_util::future;

use crate::{
    data::QouteProvider,
    models::order::{FilledOrder, OrderResult, Side},
    order::OrderReader,
};

#[derive(Debug)]
pub struct Position<'a> {
    filled_order: &'a FilledOrder,
    // TODO: consider using a different type for money
    unlrealized_profit: f64,
}

impl<'a> Position<'a> {
    pub fn new(filled_order: &'a FilledOrder, unlrealized_profit: f64) -> Self {
        Self {
            filled_order,
            unlrealized_profit,
        }
    }
}

pub struct Portfolio<'a> {
    order_reader: &'a dyn OrderReader,
    qoute_provider: &'a dyn QouteProvider,
}

impl<'a> Portfolio<'a> {
    pub fn new(order_reader: &'a dyn OrderReader, qoute_provider: &'a dyn QouteProvider) -> Self {
        Self {
            order_reader,
            qoute_provider,
        }
    }

    pub async fn get_open_positions(&self) -> Result<Vec<Position<'a>>, io::Error> {
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

                let p = Position::new(order, profit);

                Ok(p) as Result<Position, io::Error>
            })
            .collect();

        let positions = future::try_join_all(futures).await?;

        Ok(positions)
    }

    // Total portfolio value if we sold all holdings at current market rates.
    pub async fn unrealized_profit(&self) -> Result<f64, io::Error> {
        let result: f64 = self
            .get_open_positions()
            .await?
            .iter()
            .map(|p| p.unlrealized_profit)
            .sum();

        Ok(result)
    }

    pub async fn total_profit(&self) -> f64 {
        unimplemented!()
    }

    pub async fn margin_remaining(&self) -> f64 {
        unimplemented!()
    }
}
