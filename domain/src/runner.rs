use crate::event::{event::EventHandler, model::Event};
use anyhow::Result;
use core::time;
use futures_util::future;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::thread;
use std::time::Duration;

pub struct Runner {
    event_handlers: Vec<Box<dyn EventHandler + Sync + Send>>,
    data_stream: Pin<Box<dyn Stream<Item = Event>>>,
}

impl Runner {
    pub fn new(
        event_handlers: Vec<Box<dyn EventHandler + Sync + Send>>,
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

            // TODO: use https://dtantsur.github.io/rust-openstack/tokio/time/fn.delay_for.html
            let ten_millis = time::Duration::from_millis(500);
            thread::sleep(ten_millis);
        }
    }
}
