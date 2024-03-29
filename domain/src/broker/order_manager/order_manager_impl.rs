use async_trait::async_trait;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use uuid::Uuid;

use crate::broker::orders::pending::PendingKey;
use crate::{broker::Broker, models::orders::security_position::SecurityPosition};

use crate::{
    models::orders::{
        common::OrderId,
        new_order::NewOrder,
        one_cancels_others::OneCancelsOthers,
        order_result::{OrderMeta, OrderResult},
        pending_order::PendingOrder,
    },
    order::{OrderManager, OrderReader},
};

use super::utils;

#[async_trait]
impl OrderReader for Broker {
    async fn get_positions(&self) -> Result<Vec<SecurityPosition>, crate::error::Error> {
        let orders = self.orders.get_positions().await;
        Ok(orders)
    }

    async fn get_pending_orders(&self) -> Result<Vec<OrderResult>, crate::error::Error> {
        let orders = self.orders.get_pending_orders().await;
        let order_results = orders
            .iter()
            .map(|p| OrderResult::PendingOrder(p.to_owned()))
            .collect();

        Ok(order_results)
    }
}

#[async_trait]
impl OrderManager for Broker {
    async fn place_order(&self, order: &NewOrder) -> Result<OrderResult, crate::error::Error> {
        if let NewOrder::StopLimitMarket(o) = order {
            let market_order = NewOrder::Market(o.market.to_owned());
            self.place_order(&market_order).await?;

            let oco = OneCancelsOthers::builder()
                .with_quantity(o.market.order_details.quantity)
                .with_security(o.market.security.to_owned())
                .with_time_in_force(o.get_stop().times_in_force)
                .with_strategy_id(o.market.order_details.strategy_id)
                .add_limit(o.get_stop().order_details.side, o.get_stop().price)
                .add_limit(o.get_limit().order_details.side, o.get_limit().price)
                .build()
                .map_err(|e| crate::error::Error::Any(e.into()))?;

            let no = NewOrder::OCO(oco);
            return self.place_order(&no).await;
        }

        let NewOrder::Market(market_order) = order else {
            let po = PendingOrder {
                order_id: Uuid::new_v4().to_string(),
                order: order.clone(),
            };

            let or = OrderResult::PendingOrder(po.clone());

            self.orders
                .insert(&or)
                .await
                .map_err(|e| crate::error::Error::Message(e))?;

            return Ok(or);
        };

        let mut account_balance = self.account_balance.write().await;

        let (cost, filled_order) = utils::create_trade(self, market_order).await?;

        if (cost + *account_balance) < Decimal::default() {
            return Err(crate::error::Error::Message(
                "do not have enough funds to peform trade".to_string(),
            ));
        }

        let order_result = OrderResult::FilledOrder(filled_order.clone());
        self.orders
            .insert(&order_result)
            .await
            .map_err(|e| crate::error::Error::Message(e))?;
        let commision = Decimal::from_u64(market_order.order_details.quantity).unwrap()
            * self.commissions_per_share;
        let trade_cost = commision + cost;
        *account_balance += trade_cost;

        Ok(order_result)
    }

    async fn update(
        &self,
        pending_order: &PendingOrder,
    ) -> Result<OrderResult, crate::error::Error> {
        let or = OrderResult::PendingOrder(pending_order.to_owned());
        self.orders
            .insert(&or)
            .await
            .map_err(|e| crate::error::Error::Message(e))?;

        Ok(OrderResult::Updated(OrderMeta {
            order_id: pending_order.order_id.to_owned(),
            strategy_id: pending_order.startegy_id(),
        }))
    }

    async fn cancel(&self, order_id: &OrderId) -> Result<OrderResult, crate::error::Error> {
        let key = PendingKey::OrderIdKey(order_id.to_owned());
        let pending_orders = self.orders.get_pending_order(key).await;
        let pending_order = match &pending_orders[..] {
            [a] => Ok(a),
            [] => Err(crate::error::Error::Message(format!(
                "no orders associated with the order_id=`{:?}`",
                order_id
            ))),
            _ => Err(crate::error::Error::Message(format!(
                "more than one order associated with the order_id=`{:?}`",
                order_id
            ))),
        }?;

        self.orders
            .remove(&pending_order)
            .await
            .map_err(|e| crate::error::Error::Message(e))?;

        Ok(OrderResult::Updated(OrderMeta {
            order_id: order_id.to_owned(),
            strategy_id: pending_order.startegy_id(),
        }))
    }
}
