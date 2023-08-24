use rand::Rng;
use std::io;

use async_trait::async_trait;
use domain::{
    models::{
        event::Signal,
        order::{Side, TimesInForce},
        price::PriceHistory,
        security::Security,
    },
    strategy::Algorithm,
};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>, io::Error> {
        let num = rand::thread_rng().gen_range(0..100);
        if num > 50 {
            println!("fake_algo sending signal");
            let signal = Signal::new(
                "fake_algo".to_owned(),
                price_history.security.to_owned(),
                0.0,
                2000000.0,
                Side::Long,
                TimesInForce::GTC,
                0,
                0.99,
            )
            .unwrap();

            return Ok(Some(signal));
        }

        println!("fake_algo saw event");
        return Ok(None);
    }
}
