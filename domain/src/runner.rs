use crate::{
    event::{event::EventHandler, model::Event},
    models::price::PriceHistory,
};
use anyhow::Result;
use futures_util::future;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

pub trait Parser {
    fn parse(&mut self, data: &str) -> Result<Box<dyn Iterator<Item = PriceHistory>>>;
}

pub struct Runner {
    event_handlers: Vec<Box<dyn EventHandler>>,
    data_stream: Pin<Box<dyn Stream<Item = Event>>>,
}

impl Runner {
    pub fn new(
        event_handlers: Vec<Box<dyn EventHandler>>,
        data_stream: Pin<Box<dyn Stream<Item = Event>>>,
    ) -> Self {
        Self {
            event_handlers,
            data_stream,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(event) = self.data_stream.next().await {
                let futures = self
                    .event_handlers
                    .iter()
                    .map(|eh| async { eh.handle(&event).await });
                future::try_join_all(futures).await?;
            }

            // TODO:! sleep for 500 millis if no tasks
        }
    }
}
