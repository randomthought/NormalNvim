use std::time::{SystemTime, UNIX_EPOCH};

use actix::Actor;
use async_trait::async_trait;
use color_eyre::eyre::Result;
use domain::{
    event::{self, model},
    models::orders::{
        common::{OrderDetails, Side},
        market::Market,
        new_order::NewOrder,
    },
    strategy::{
        algorithm::{Algorithm, StrategyId},
        model::{
            algo_event::AlgoEvent,
            signal::{Entry, Signal},
        },
    },
};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    fn strategy_id(&self) -> StrategyId {
        "fake_algo".into()
    }
    async fn on_event(
        &self,
        algo_event: AlgoEvent,
    ) -> Result<Option<Signal>, domain::error::Error> {
        if let AlgoEvent::OrderResult(order_result) = algo_event {
            println!("fake_algo: my order was filled: {:?}", order_result);
            return Ok(None);
        };

        let AlgoEvent::DataEvent(event::model::DataEvent::PriceEvent(price_history)) = algo_event
        else {
            return Ok(None);
        };
        // println!("fake_algo saw event");

        // let mut rng = StdRng::seed_from_u64(4);
        // let rm = rng.gen_range(0.0..1.0);

        let rm = rand::thread_rng().gen_range(0.0..1.0);
        if rm <= 0.02 {
            let rm2 = rand::thread_rng().gen_range(0.0..1.0);
            if rm2 < 0.05 {
                println!("fake_algo liquidate signal");
                return Ok(Some(Signal::Liquidate(self.strategy_id())));
            }
            println!("fake_algo sending signal");
            let security = price_history.security.to_owned();
            let market = Market::builder()
                .with_security(security)
                .with_order_details(
                    OrderDetails::builder()
                        .with_strategy_id(self.strategy_id())
                        .with_quantity(1)
                        .with_side(Side::Long)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap();
            let order = NewOrder::Market(market);
            let datetime = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| domain::error::Error::Any(e.into()))?;

            // let signal = Signal::new(
            let signal = Signal::Entry(Entry::new(order, datetime, 0.99));
            return Ok(Some(signal));
        }

        return Ok(None);
    }
}
