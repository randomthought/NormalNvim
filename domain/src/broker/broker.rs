use crate::{
    data::QouteProvider,
    models::{
        orders::{
            common::Side, filled_order::FilledOrder, market::Market,
            security_position::SecurityPosition,
        },
        price::{common::Price, quote::Quote},
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
}

// #[async_trait]
// impl EventHandler for Broker {
//     async fn handle(&self, event: &Event) -> Result<(), crate::error::Error> {
//         let Event::Market(event::model::Market::DataEvent(d)) = event else {
//             return Ok(());
//         };
//
//         let Some(candle) = d.history.last() else {
//             return Ok(());
//         };
//
//         let security = &d.security;
//         let pending = self.orders.get_pending_order(security).await;
//
//         for p in pending {
//             match p.order {
//                 NewOrder::Limit(o) => {
//                     let met = match o.order_details.side {
//                         Side::Long => o.price >= candle.close,
//                         Side::Short => o.price <= candle.close,
//                     };
//                     if !met {
//                         continue;
//                     }
//
//                     // TODO: with this implementation, you would not get the exact limit price
//                     let m = Market::new(
//                         o.order_details.quantity,
//                         o.order_details.side,
//                         o.security.to_owned(),
//                         o.strategy_id(),
//                     );
//                     let order = NewOrder::Market(m);
//                     self.place_order(&order).await?;
//                 }
//                 NewOrder::StopLimitMarket(o) => todo!(),
//                 _ => continue,
//             };
//         }
//
//         todo!()
//     }
// }

fn create_filled_order(
    quantity: u64,
    security: &Security,
    side: Side,
    quote: &Quote,
    strategy_id: StrategyId,
) -> Result<FilledOrder, crate::error::Error> {
    let price = match side {
        Side::Long => quote.ask,
        Side::Short => quote.bid,
    };

    let order_id = Uuid::new_v4().to_string();

    let datetime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| crate::error::Error::Any(e.into()))?;

    let fo = FilledOrder::builder()
        .with_order_id(order_id)
        .with_date_time(datetime)
        .with_price(price)
        .with_security(security.to_owned())
        .with_quantity(quantity)
        .with_side(side)
        .with_strategy_id(strategy_id)
        .build()
        .map_err(|e| crate::error::Error::Any(e.into()))?;

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
