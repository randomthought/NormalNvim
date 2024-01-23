use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use color_eyre::eyre::Result;
use domain::{
    event::model::Signal,
    models::{
        order::{self, FilledOrder, OrderResult, TimesInForce},
        price::PriceHistory,
    },
    strategy::Algorithm,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    fn get_id(&self) -> String {
        "fake_algo".into()
    }
    async fn on_data(&self, price_history: &PriceHistory) -> Result<Option<Signal>> {
        // println!("fake_algo saw event");

        // let mut rng = StdRng::seed_from_u64(4);
        // let rm = rng.gen_range(0.0..1.0);

        let rm = rand::thread_rng().gen_range(0.0..1.0);
        if rm <= 0.01 {
            println!("fake_algo sending signal");
            let security = price_history.security.to_owned();
            let market = order::Market::new(1, order::Side::Long, security);
            let order = order::NewOrder::Market(market);
            let strategy_id = "fake_algo".to_owned();
            let datetime = SystemTime::now().duration_since(UNIX_EPOCH)?;
            let signal = Signal::new(strategy_id, order, datetime, 0.99);

            return Ok(Some(signal));
        }

        return Ok(None);
    }

    async fn on_order(&self, order_result: &OrderResult) -> Result<()> {
        println!("fake_algo: my order was filled: {:?}", order_result);
        return Ok(());
    }
}
