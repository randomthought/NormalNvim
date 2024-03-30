use rust_decimal::{prelude::FromPrimitive, Decimal};
use uuid::Uuid;

use crate::{
    broker::Broker,
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
use std::time::{SystemTime, UNIX_EPOCH};

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

pub async fn create_trade(
    broker: &Broker,
    market_order: &Market,
) -> Result<(Price, FilledOrder), crate::error::Error> {
    let quote = broker
        .qoute_provider
        .get_quote(&market_order.security)
        .await?;

    let price = match market_order.order_details.side {
        Side::Long => quote.bid,
        Side::Short => quote.ask,
    };
    let Some(active) = broker.orders.get_position(&market_order.security).await else {
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
