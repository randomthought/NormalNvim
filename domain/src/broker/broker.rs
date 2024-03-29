use crate::{
    broker::orders::pending::PendingKey,
    data::QouteProvider,
    models::{
        orders::{
            common::Side, filled_order::FilledOrder, market::Market,
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

    pub async fn handle(&self, candle: &Candle) {
        let pending_key = PendingKey::SecurityKey(candle.security.clone());
        let pending_orders = self.orders.get_pending_order(pending_key).await;

        for p in pending_orders.iter() {
            // match p.order {
            //     NewOrder::Limit(o) => {
            //         let met = match o.order_details.side {
            //             Side::Long => o.price >= candle.close,
            //             Side::Short => o.price <= candle.close,
            //         };
            //         if !met {
            //             continue;
            //         }
            //
            //         // TODO: with this implementation, you would not get the exact limit price
            //         let m = Market::new(
            //             o.order_details.quantity,
            //             o.order_details.side,
            //             o.security.to_owned(),
            //             o.strategy_id(),
            //         );
            //         let order = NewOrder::Market(m);
            //         self.place_order(&order).await?;
            //     }
            //     NewOrder::StopLimitMarket(o) => todo!(),
            //     _ => continue,
            // };
        }
        todo!()
    }
}
