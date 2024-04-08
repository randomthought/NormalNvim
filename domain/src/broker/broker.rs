use crate::broker::{orders::pending::PendingKey, utils};
use models::{
    orders::{
        common::Side, limit::Limit, market::Market, new_order::NewOrder, order_result::OrderResult,
    },
    price::{price_bar::PriceBar, quote::Quote},
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::sync::Arc;
use tokio::sync::RwLock;
use traits::{data::QouteProvider, order::OrderManager};

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

    pub async fn handle(
        &self,
        candle: &PriceBar,
    ) -> Result<Vec<OrderResult>, models::error::Error> {
        let pending_key = PendingKey::SecurityKey(candle.security.clone());
        let pending_orders = self.orders.get_pending_order(pending_key).await;

        let mut order_results = vec![];
        for p in pending_orders.iter() {
            match p.order() {
                NewOrder::Limit(o) => {
                    let results = self.handle_limit(&o, candle).await?;
                    if let Some(v) = results {
                        self.cancel(p.order_id()).await?;
                        order_results.push(v);
                    }
                }
                NewOrder::OCO(o) => {
                    for limit in o.orders.iter() {
                        let results = self.handle_limit(limit, candle).await?;
                        if let Some(v) = results {
                            self.cancel(p.order_id()).await?;
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
        candle: &PriceBar,
    ) -> Result<Option<OrderResult>, models::error::Error> {
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
            .map_err(|e| models::error::Error::Any(e.into()))?;

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
