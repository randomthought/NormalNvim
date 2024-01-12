#[cfg(test)]
use core::panic;
use std::time::{SystemTime, UNIX_EPOCH};

use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    broker::active_order::ActiveOrder,
    models::{
        order::{FilledOrder, HoldingDetail, SecurityPosition, Side},
        security::{AssetType, Exchange, Security},
    },
};

#[test]
fn empty_active_order() {
    let security = Security::new(AssetType::Equity, Exchange::NYSE, "GE".into());

    let mut active_order = ActiveOrder::new(security.to_owned());

    assert!(
        active_order.get_position().is_none(),
        "nothing inserted into active order"
    );
}

#[test]
fn insert() {
    let security = Security::new(AssetType::Equity, Exchange::NYSE, "GE".into());

    let mut active_order = ActiveOrder::new(security.to_owned());

    let price = Decimal::from_f32(100.0).unwrap();
    let filled_order = FilledOrder::new(
        security.to_owned(),
        "fake_order_id".into(),
        price,
        20,
        Side::Long,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );

    active_order.insert(&filled_order).unwrap();

    assert!(
        active_order.get_position().is_some(),
        "inserted a value into active order"
    );
}

#[test]
fn increase_position() {
    let security = Security::new(AssetType::Equity, Exchange::NYSE, "GE".into());

    let mut active_order = ActiveOrder::new(security.to_owned());

    let price = Decimal::from_f32(100.0).unwrap();
    let filled_order_1 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_1".into(),
        price,
        10,
        Side::Long,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );
    let filled_order_2 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_2".into(),
        price,
        20,
        Side::Long,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );

    active_order.insert(&filled_order_1).unwrap();
    active_order.insert(&filled_order_2).unwrap();

    let Some(result) = active_order.get_position() else {
        panic!("security position is none")
    };

    let expected = SecurityPosition {
        security: security.to_owned(),
        side: Side::Long,
        holding_details: vec![
            HoldingDetail {
                price,
                quantity: 10,
            },
            HoldingDetail {
                price,
                quantity: 20,
            },
        ],
    };

    assert_eq!(result, expected);
}

#[test]
fn close_position() {
    let security = Security::new(AssetType::Equity, Exchange::NYSE, "GE".into());

    let mut active_order = ActiveOrder::new(security.to_owned());

    let price = Decimal::from_f32(100.0).unwrap();
    let filled_order_1 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_1".into(),
        price,
        10,
        Side::Long,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );
    let filled_order_2 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_2".into(),
        price,
        20,
        Side::Long,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );
    let filled_order_3 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_2".into(),
        price,
        30,
        Side::Short,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );

    active_order.insert(&filled_order_1).unwrap();
    active_order.insert(&filled_order_2).unwrap();
    active_order.insert(&filled_order_3).unwrap();
    println!("{:?}", active_order.get_position());

    let results = active_order.get_position();

    assert!(
        results.is_none(),
        "position exist when it should have been closed"
    )
}

#[test]
fn flip_position() {
    let security = Security::new(AssetType::Equity, Exchange::NYSE, "GE".into());

    let mut active_order = ActiveOrder::new(security.to_owned());

    let price = Decimal::from_f32(100.0).unwrap();
    let filled_order_1 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_1".into(),
        price,
        10,
        Side::Long,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );
    let filled_order_2 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_2".into(),
        price,
        20,
        Side::Long,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );
    let filled_order_3 = FilledOrder::new(
        security.to_owned(),
        "fake_order_id_2".into(),
        price,
        40,
        Side::Short,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
    );

    active_order.insert(&filled_order_1).unwrap();
    active_order.insert(&filled_order_2).unwrap();
    active_order.insert(&filled_order_3).unwrap();

    let Some(result) = active_order.get_position() else {
        panic!("security position is none")
    };

    let expected = SecurityPosition {
        security: security.to_owned(),
        side: Side::Short,
        holding_details: vec![HoldingDetail {
            price,
            quantity: 10,
        }],
    };

    assert_eq!(expected, result);
}
