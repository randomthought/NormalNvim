use super::{
    orders::security_transaction::{SecurityTransaction, Transaction},
    Broker,
};
use crate::{
    models::orders::{
        common::{OrderDetails, Side},
        pending_order::PendingOrder,
        security_position::SecurityPosition,
    },
    order::OrderReader,
    strategy::{algorithm::StrategyId, portfolio::StrategyPortfolio},
};
use async_trait::async_trait;
use rust_decimal::{prelude::FromPrimitive, Decimal};

#[async_trait]
impl StrategyPortfolio for Broker {
    async fn get_profit(&self, strategy_id: StrategyId) -> Result<Decimal, crate::error::Error> {
        let security_transactions = self
            .orders
            .get_transactions()
            .await
            .map_err(|e| crate::error::Error::Message(e))?;

        let result = security_transactions
            .iter()
            .map(|st| calculate_profit(st, strategy_id))
            .sum();

        Ok(result)
    }

    async fn get_security_positions(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<SecurityPosition>, crate::error::Error> {
        let open_positions = self.get_positions().await?;
        // TODO: this could cause issues. especially imformation conflict if algos are trading the same instruments
        let algo_positions: Vec<_> = open_positions
            .iter()
            .flat_map(|p| {
                p.holding_details
                    .to_owned()
                    .into_iter()
                    .filter(|h| h.strategy_id == strategy_id)
                    .fold(SecurityPosition::builder(), |mut spb, hd| {
                        spb.add_holding_detail(hd).to_owned()
                    })
                    .with_security(p.security.to_owned())
                    .with_side(p.side)
                    .build()
                    .ok()
            })
            .collect();

        Ok(algo_positions)
    }

    async fn get_pending(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<PendingOrder>, crate::error::Error> {
        let pending = self.orders.get_pending_orders().await;

        let algo_pending: Vec<_> = pending
            .iter()
            .filter(|p| p.startegy_id() == strategy_id)
            .map(|p| p.to_owned())
            .collect();

        Ok(algo_pending)
    }
}

fn _calucluate_profit(large: &Transaction, small: &Transaction) -> (Decimal, Option<Transaction>) {
    let q_remaining = large.order_details.quantity - small.order_details.quantity;

    let sq = Decimal::from_u64(small.order_details.quantity).unwrap();
    let profit = match small.order_details.side {
        Side::Long => sq * (large.price - small.price),
        Side::Short => sq * (small.price - large.price),
    };

    if q_remaining == 0 {
        return (profit, None);
    }

    let t = Transaction {
        order_details: OrderDetails {
            quantity: q_remaining,
            ..large.order_details
        },
        ..large.to_owned()
    };

    (profit, Some(t))
}

fn calculate_profit(
    security_transaction: &SecurityTransaction,
    strategy_id: StrategyId,
) -> Decimal {
    let algo_transaction: Vec<_> = security_transaction
        .order_history
        .iter()
        .filter(|t| t.order_details.strategy_id == strategy_id)
        .collect();

    let (profit, ots) = algo_transaction.iter().map(|t| t.to_owned()).fold(
        (Decimal::default(), None),
        |(pf, c), n| {
            let Some(current) = c else {
                return (pf, Some(n.to_owned()));
            };

            match (current.order_details.side, n.order_details.side) {
                (Side::Long, Side::Short) => {
                    if n.order_details.quantity > current.order_details.quantity {
                        _calucluate_profit(n, &current)
                    } else {
                        _calucluate_profit(&current, n)
                    }
                }
                (Side::Short, Side::Long) => {
                    if n.order_details.quantity > current.order_details.quantity {
                        _calucluate_profit(n, &current)
                    } else {
                        _calucluate_profit(&current, n)
                    }
                }
                _ => {
                    let quantity = current.order_details.quantity + n.order_details.quantity;
                    let c_quantity = Decimal::from_u64(current.order_details.quantity).unwrap();
                    let n_quantity = Decimal::from_u64(n.order_details.quantity).unwrap();
                    let price = ((c_quantity * current.price) + (n_quantity * n.price))
                        / Decimal::from_u64(quantity).unwrap();
                    let t = Transaction {
                        order_details: OrderDetails {
                            quantity,
                            ..n.order_details
                        },
                        price,
                        order_id: n.order_id.to_owned(),
                        date_time: n.date_time.to_owned(),
                    };

                    (pf, Some(t))
                }
            }
        },
    );

    if let Some(_) = ots {
        return Decimal::default();
    }

    profit
}
