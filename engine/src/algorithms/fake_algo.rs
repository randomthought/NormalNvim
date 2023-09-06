use anyhow::{Context, Result};
use async_trait::async_trait;
use domain::{
    models::{
        event::Signal,
        order::{Side, TimesInForce},
        price::PriceHistory,
    },
    strategy::Algorithm,
};
use rand::Rng;
use rust_decimal::{prelude::FromPrimitive, Decimal};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>> {
        let num = rand::thread_rng().gen_range(0..100);
        if num > 50 {
            println!("fake_algo sending signal");
            let signal = Signal::new(
                "fake_algo".to_owned(),
                price_history.security.to_owned(),
                Decimal::from_f64(0.0).unwrap(),
                Decimal::from_f64(2000000.0).unwrap(),
                Side::Long,
                TimesInForce::GTC,
                0,
                0.99,
            )?;

            return Ok(Some(signal));
        }

        println!("fake_algo saw event");
        return Ok(None);
    }
}
