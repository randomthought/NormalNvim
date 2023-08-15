use std::io;

use async_trait::async_trait;
use domain::{
    models::{event::Signal, price::PriceHistory},
    strategy::Algorithm,
};

pub struct FakeAlgo {}

#[async_trait]
impl Algorithm for FakeAlgo {
    async fn process(&self, price_history: &PriceHistory) -> Result<Option<Signal>, io::Error> {
        todo!()
    }
}
