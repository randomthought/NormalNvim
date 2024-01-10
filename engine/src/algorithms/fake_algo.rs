use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use domain::{
    event::model::Signal,
    models::{
        order::{self, TimesInForce},
        price::PriceHistory,
    },
    strategy::Algorithm,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>> {
        // println!("fake_algo saw event");

        // let mut rng = StdRng::seed_from_u64(4);
        // let rm = rng.gen_range(0.0..1.0);

        let rm = rand::thread_rng().gen_range(0.0..1.0);
        if rm <= 0.0001 {
            println!("fake_algo sending signal");
            let security = price_history.security.to_owned();
            let market = order::Market::new(1, order::Side::Long, security);
            let order = order::Order::Market(market);
            let strategy_id = "fake_algo".to_owned();
            let datetime = SystemTime::now().duration_since(UNIX_EPOCH)?;
            let signal = Signal::new(strategy_id, order, datetime, 0.99);

            return Ok(Some(signal));
        }

        return Ok(None);
    }
}
