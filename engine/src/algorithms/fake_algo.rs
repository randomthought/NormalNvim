use rand::Rng;
use std::io;

use async_trait::async_trait;
use domain::{
    models::{event::Signal, order::Side, price::PriceHistory, security::Security},
    strategy::Algorithm,
};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>, io::Error> {
        let num = rand::thread_rng().gen_range(0..100);
        if num > 50 {
            println!("fake_algo sending signal");
            let signal = Signal {
                strategy_id: "fake_algo".to_owned(),
                security: price_history.security.to_owned(),
                side: Side::Long,
                datetime: 0,
                strength: 0.99,
            };

            return Ok(Some(signal));
        }

        println!("fake_algo saw event");
        return Ok(None);
    }
}
