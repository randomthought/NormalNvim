use crate::{
    broker::{orders::pending::PendingKey, utils},
    data::QouteProvider,
    models::{
        orders::{
            common::Side, filled_order::FilledOrder, limit::Limit, market::Market,
            new_order::NewOrder, order_result::OrderResult, pending_order::PendingOrder,
            security_position::SecurityPosition,
        },
        price::{candle::Candle, common::Price, quote::Quote},
        security::Security,
    },
    strategy::algorithm::StrategyId,
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, u64};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::orders::orders::Orders;

pub struct Broker {
    pub(super) qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
    pub(super) account_balance: RwLock<Decimal>,
    pub(super) orders: Orders,
    pub(super) commissions_per_share: Decimal,
}

impl Broker {
    pub fn new(
        account_balance: Decimal,
        qoute_provider: Arc<dyn QouteProvider + Sync + Send>,
    ) -> Self {
        let commissions_per_share = Decimal::from_f64(0.0).unwrap();
        Self {
            account_balance: RwLock::new(account_balance),
            commissions_per_share,
            orders: Orders::new(),
            qoute_provider,
        }
    }

    pub async fn handle(&self, candle: &Candle) -> Result<Vec<OrderResult>, crate::error::Error> {
        let pending_key = PendingKey::SecurityKey(candle.security.clone());
        let pending_orders = self.orders.get_pending_order(pending_key).await;

        let mut order_results = vec![];
        for p in pending_orders.iter() {
            match p.order() {
                NewOrder::Limit(o) => {
                    let results = self.handle_limit(&o, candle).await?;
                    if let Some(v) = results {
                        order_results.push(v);
                    }
                }
                NewOrder::OCO(o) => {
                    for limit in o.orders.iter() {
                        let results = self.handle_limit(limit, candle).await?;
                        if let Some(v) = results {
                            self.orders
                                .remove(p)
                                .await
                                .map_err(|e| crate::error::Error::Any(e.into()))?;
                            order_results.push(v);
                            break;
                        }
                    }
                }
                _ => continue,
            };
        }

        Ok(order_results)
    }

    async fn handle_limit(
        &self,
        limit: &Limit,
        candle: &Candle,
    ) -> Result<Option<OrderResult>, crate::error::Error> {
        let met = match limit.order_details.side() {
            Side::Long => limit.price >= candle.close,
            Side::Short => limit.price <= candle.close,
        };

        if !met {
            return Ok(None);
        }

        let quote = Quote::builder()
            .with_security(candle.security.clone())
            .with_timestamp(candle.start_time)
            .with_bid_size(1)
            .with_ask_size(1)
            .with_bid(limit.price)
            .with_ask(limit.price)
            .build()
            .map_err(|e| crate::error::Error::Any(e.into()))?;

        let market_order = Market::builder()
            .with_security(limit.security.clone())
            .with_side(limit.order_details.side())
            .with_quantity(limit.order_details.quantity())
            .with_strategy_id(limit.order_details.strategy_id())
            .build()
            .unwrap();

        let order_result = utils::execute_market_order(self, &quote, &market_order).await?;

        Ok(Some(order_result))
    }
}
