use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use color_eyre::eyre::Result;
use domain::{
    event::{
        self,
        model::{Market, Signal},
    },
    models::order::{self},
    strategy::{
        algo_event::AlgoEvent,
        algorithm::{Algorithm, StrategyId},
    },
};
use eyre::Ok;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    fn strategy_id(&self) -> StrategyId {
        "fake_algo".into()
    }
    async fn on_event<'a>(&self, algo_event: AlgoEvent<'a>) -> Result<Option<Signal>> {
        if let AlgoEvent::OrderResult(order_result) = algo_event {
            println!("fake_algo: my order was filled: {:?}", order_result);
            return Ok(None);
        };

        let AlgoEvent::Market(Market::DataEvent(price_history)) = algo_event else {
            return Ok(None);
        };
        // println!("fake_algo saw event");

        // let mut rng = StdRng::seed_from_u64(4);
        // let rm = rng.gen_range(0.0..1.0);

        let rm = rand::thread_rng().gen_range(0.0..1.0);
        if rm <= 0.01 {
            // println!("fake_algo sending signal");
            let security = price_history.security.to_owned();
            let market = order::Market::new(1, order::Side::Long, security, self.strategy_id());
            let order = order::NewOrder::Market(market);
            let datetime = SystemTime::now().duration_since(UNIX_EPOCH)?;
            // let signal = Signal::new(
            let signal = Signal::Entry(event::model::Entry::new(order, datetime, 0.99));
            return Ok(Some(signal));
        }

        return Ok(None);
    }
}
