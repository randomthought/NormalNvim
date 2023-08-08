use std::io;

use futures_lite::{stream, StreamExt};

use crate::{
    data::DataProvider,
    models::order::{FilledOrder, OrderResult, Side},
    order::OrderReader,
};

#[derive(Debug)]
pub struct Position<'a> {
    filled_order: &'a FilledOrder,
    // TODO: consider using a different type for money
    unlrealized_profit: f32,
}

impl<'a> Position<'a> {
    pub fn new(filled_order: &'a FilledOrder, unlrealized_profit: f32) -> Self {
        Self {
            filled_order,
            unlrealized_profit,
        }
    }
}

pub struct Portfolio<'a> {
    order_reader: &'a dyn OrderReader,
    data_provider: &'a dyn DataProvider,
}

impl<'a> Portfolio<'a> {
    pub fn new(order_reader: &'a dyn OrderReader, data_provider: &'a dyn DataProvider) -> Self {
        Self {
            order_reader,
            data_provider,
        }
    }

    pub async fn get_open_positions(&self) -> Result<Vec<Position<'a>>, io::Error> {
        let orders = self.order_reader.orders().await.unwrap();

        // let positions: Vec<Position<'a>> = stream::iter(orders)
        let positions: Vec<Position<'a>> = stream::iter(orders)
            .then(|order| async move {
                match order {
                    OrderResult::FilledOrder(fo) => {
                        let quote = self.data_provider.get_quote(&fo.security).await.unwrap();
                        let profit = match fo.side {
                            Side::Long => fo.price - quote.ask,
                            Side::Short => quote.bid - fo.price,
                        };

                        let p: Position<'a> = Position::new(fo, profit);
                        Some(p)
                    }

                    _ => None,
                }
            })
            .filter_map(|x| x)
            .collect()
            .await;

        Ok(positions)
    }

    // Total portfolio value if we sold all holdings at current market rates.
    pub async fn unrealized_profit(&self) -> Result<f32, String> {
        let result: f32 = self
            .get_open_positions()
            .await
            .unwrap()
            .iter()
            .map(|p| p.unlrealized_profit)
            .sum();

        Ok(result)
    }

    pub async fn total_profit(&self) -> f32 {
        unimplemented!()
    }

    pub async fn margin_remaining(&self) -> f32 {
        unimplemented!()
    }
}
