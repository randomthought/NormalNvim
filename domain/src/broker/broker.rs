use crate::{
    broker::security_transaction::Transation,
    data::QouteProvider,
    event::{
        self,
        event::{EventHandler, EventProducer},
        model::Event,
    },
    models::{
        order::{
            self, FilledOrder, HoldingDetail, NewOrder, OrderMeta, OrderResult, PendingOrder,
            SecurityPosition,
        },
        price::{Price, Quote},
        security::Security,
    },
    order::{Account, OrderManager, OrderReader},
    strategy::{algorithm::StrategyId, portfolio::StrategyPortfolio},
};
use async_trait::async_trait;
use color_eyre::eyre::{bail, Ok, Result};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, u64};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{orders::Orders, security_transaction::SecurityTransaction};

pub struct Broker {
    event_producer: Arc<dyn EventProducer + Sync + Send>,
    qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
    account_balance: RwLock<Decimal>,
    orders: Orders,
    commissions_per_share: Decimal,
}

impl Broker {
    pub fn new(
        account_balance: Decimal,
        qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
        event_producer: Arc<dyn EventProducer + Sync + Send>,
    ) -> Self {
        let commissions_per_share = Decimal::from_f64(0.0).unwrap();
        Self {
            event_producer,
            account_balance: RwLock::new(account_balance),
            commissions_per_share,
            orders: Orders::new(),
            qoute_provider,
        }
    }

    async fn create_trade(&self, market_order: &order::Market) -> Result<(Price, FilledOrder)> {
        let quote = self
            .qoute_provider
            .get_quote(&market_order.security)
            .await?;

        let price = match market_order.order_details.side {
            order::Side::Long => quote.bid,
            order::Side::Short => quote.ask,
        };
        let Some(active) = self.orders.get_position(&market_order.security).await else {
            let cost = Decimal::from_u64(market_order.order_details.quantity).unwrap() * -price;
            let filled_order = create_filled_order(
                market_order.order_details.quantity,
                &market_order.security,
                market_order.order_details.side,
                &quote,
                market_order.startegy_id(),
            )?;
            return Ok((cost, filled_order));
        };

        if active.side == market_order.order_details.side {
            let filled_order = create_filled_order(
                market_order.order_details.quantity,
                &market_order.security,
                market_order.order_details.side,
                &quote,
                market_order.startegy_id(),
            )?;
            let cost = calculate_cost(&active, &filled_order);
            return Ok((cost, filled_order));
        }

        let active_position_quantity = active.get_quantity();
        if active_position_quantity == market_order.order_details.quantity {
            let filled_order = create_filled_order(
                market_order.order_details.quantity,
                &market_order.security,
                market_order.order_details.side,
                &quote,
                market_order.startegy_id(),
            )?;

            let cost = calculate_cost(&active, &filled_order);
            return Ok((cost, filled_order));
        }

        let side = if active_position_quantity > market_order.order_details.quantity {
            active.side
        } else {
            market_order.order_details.side
        };

        let filled_order = create_filled_order(
            market_order.order_details.quantity,
            &market_order.security,
            side,
            &quote,
            market_order.startegy_id(),
        )?;

        let cost = calculate_cost(&active, &filled_order);
        return Ok((cost, filled_order));
    }
}

#[async_trait]
impl Account for Broker {
    async fn get_account_balance(&self) -> Result<Decimal> {
        let balance = self.account_balance.read().await;
        Ok(*balance)
    }
    async fn get_buying_power(&self) -> Result<Decimal> {
        let balance = self.account_balance.read().await;
        Ok(*balance)
    }
}

#[async_trait]
impl OrderReader for Broker {
    async fn get_positions(&self) -> Result<Vec<SecurityPosition>> {
        let orders = self.orders.get_positions().await;
        Ok(orders)
    }

    async fn get_pending_orders(&self) -> Result<Vec<OrderResult>> {
        let orders = self.orders.get_pending_orders().await;
        let order_results = orders
            .iter()
            .map(|p| OrderResult::PendingOrder(p.to_owned()))
            .collect();

        Ok(order_results)
    }
}

fn _calucluate_profit(large: &Transation, small: &Transation) -> (Decimal, Transation) {
    let q_remaining = large.order_details.quantity - small.order_details.quantity;
    let t = Transation {
        order_details: order::OrderDetails {
            quantity: q_remaining,
            ..large.order_details
        },
        ..large.to_owned()
    };

    let sq = Decimal::from_u64(small.order_details.quantity).unwrap();
    let profit = match small.order_details.side {
        order::Side::Long => sq * (large.price - small.price),
        order::Side::Short => sq * (small.price - large.price),
    };

    (profit, t)
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

    let long_quantity = algo_transaction
        .iter()
        .filter(|t| matches!(t.order_details.side, order::Side::Long))
        .fold(0u64, |acc, n| acc + n.order_details.quantity);

    let short_quantity = algo_transaction
        .iter()
        .filter(|t| matches!(t.order_details.side, order::Side::Short))
        .fold(0u64, |acc, n| acc + n.order_details.quantity);

    if long_quantity != short_quantity {
        return Decimal::default();
    }

    let (profit, _) = algo_transaction.iter().map(|t| t.to_owned()).fold(
        (Decimal::default(), None),
        |(pf, c), n| {
            let Some(current) = c else {
                return (pf, Some(n.to_owned()));
            };

            let (p, t) = match (current.order_details.side, n.order_details.side) {
                (order::Side::Long, order::Side::Short) => {
                    if n.order_details.quantity > current.order_details.quantity {
                        _calucluate_profit(n, &current)
                    } else {
                        _calucluate_profit(&current, n)
                    }
                }
                (order::Side::Short, order::Side::Long) => {
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
                    let t = Transation {
                        order_details: order::OrderDetails {
                            quantity,
                            ..n.order_details
                        },
                        price,
                        order_id: n.order_id.to_owned(),
                        date_time: n.date_time.to_owned(),
                    };

                    (pf, t)
                }
            };

            (p, Some(t))
        },
    );

    profit
}

#[async_trait]
impl StrategyPortfolio for Broker {
    async fn get_profit(&self, strategy_id: StrategyId) -> Result<Decimal> {
        let security_transactions = self.orders.get_transactions().await?;
        let result = security_transactions
            .iter()
            .map(|st| calculate_profit(st, strategy_id))
            .sum();

        Ok(result)
    }

    async fn get_holdings(&self, strategy_id: StrategyId) -> Result<Vec<SecurityPosition>> {
        let open_positions = self.get_positions().await?;
        // TODO: this could cause issues. especially imformation conflict if algos are trading the same instruments
        let algo_positions: Vec<_> = open_positions
            .iter()
            .flat_map(|p| {
                let holding_details: Vec<HoldingDetail> = p
                    .holding_details
                    .iter()
                    .filter(|h| h.strategy_id == strategy_id)
                    .map(|h| h.to_owned())
                    .collect();

                if holding_details.is_empty() {
                    return None;
                }

                Some(SecurityPosition {
                    holding_details,
                    security: p.security.to_owned(),
                    side: p.side,
                })
            })
            .collect();

        Ok(algo_positions)
    }
}

#[async_trait]
impl OrderManager for Broker {
    async fn place_order(&self, order: &NewOrder) -> Result<OrderResult> {
        if let NewOrder::StopLimitMarket(o) = order {
            let market_order = NewOrder::Market(o.market.to_owned());
            self.place_order(&market_order).await?;

            let oca = NewOrder::OCA(o.one_cancels_other.to_owned());
            return self.place_order(&oca).await;
        }

        let NewOrder::Market(market_order) = order else {
            let po = order::PendingOrder {
                order_id: Uuid::new_v4().to_string(),
                order: order.clone(),
            };

            let or = order::OrderResult::PendingOrder(po.clone());

            self.orders.insert(&or).await?;

            return Ok(or);
        };

        let mut account_balance = self.account_balance.write().await;

        let (cost, filled_order) = self.create_trade(market_order).await?;

        if (cost + *account_balance) < Decimal::new(0, 0) {
            bail!("do not have enough funds to peform trade");
        }

        let order_result = order::OrderResult::FilledOrder(filled_order.clone());
        self.orders.insert(&order_result).await?;
        let commision = Decimal::from_u64(market_order.order_details.quantity).unwrap()
            * self.commissions_per_share;
        let trade_cost = commision + cost;
        *account_balance += trade_cost;

        Ok(order_result)
    }

    async fn update(&self, pending_order: &PendingOrder) -> Result<OrderResult> {
        let or = order::OrderResult::PendingOrder(pending_order.to_owned());
        self.orders.insert(&or).await?;

        Ok(OrderResult::Updated(OrderMeta {
            order_id: pending_order.order_id.to_owned(),
            strategy_id: pending_order.startegy_id(),
        }))
    }

    async fn cancel(&self, pending_order: &PendingOrder) -> Result<OrderResult> {
        self.orders.remove(&pending_order).await?;
        Ok(OrderResult::Updated(OrderMeta {
            order_id: pending_order.order_id.to_owned(),
            strategy_id: pending_order.startegy_id(),
        }))
    }
}

#[async_trait]
impl EventHandler for Broker {
    async fn handle(&self, event: &Event) -> Result<()> {
        let Event::Market(event::model::Market::DataEvent(d)) = event else {
            return Ok(());
        };

        let Some(candle) = d.history.last() else {
            return Ok(());
        };

        let security = &d.security;
        let pending = self.orders.get_pending_order(security).await;

        for p in pending {
            match p.order {
                NewOrder::Limit(o) => {
                    let met = match o.order_details.side {
                        order::Side::Long => o.price >= candle.close,
                        order::Side::Short => o.price <= candle.close,
                    };
                    if !met {
                        continue;
                    }

                    // TODO: with this implementation, you would not get the exact limit price
                    let m = order::Market::new(
                        o.order_details.quantity,
                        o.order_details.side,
                        o.security.to_owned(),
                        o.strategy_id(),
                    );
                    let order = NewOrder::Market(m);
                    self.place_order(&order).await?;
                }
                NewOrder::StopLimitMarket(o) => todo!(),
                _ => continue,
            };
        }

        todo!()
    }
}

fn create_filled_order(
    quantity: u64,
    security: &Security,
    side: order::Side,
    quote: &Quote,
    strategy_id: StrategyId,
) -> Result<FilledOrder> {
    let price = match side {
        order::Side::Long => quote.ask,
        order::Side::Short => quote.bid,
    };

    let order_id = Uuid::new_v4().to_string();

    let datetime = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let fo = FilledOrder::new(
        security.to_owned(),
        order_id,
        price,
        quantity,
        side,
        datetime,
        strategy_id,
    );

    Ok(fo)
}

fn calculate_cost(security_position: &SecurityPosition, filled_order: &FilledOrder) -> Price {
    let quantity = if security_position.side == filled_order.order_details.side {
        -Decimal::from_u64(filled_order.order_details.quantity).unwrap()
    } else {
        Decimal::from_u64(filled_order.order_details.quantity).unwrap()
    };

    quantity * filled_order.price
}
