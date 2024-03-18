use crate::event::{event::EventHandler, model::Event};
use futures_util::future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::{Stream, StreamExt};

pub struct Runner {
    event_handlers: Vec<Box<dyn EventHandler + Sync + Send>>,
    event_stream: Pin<Box<dyn Stream<Item = Event> + Sync + Send>>,
}

impl Runner {
    pub fn new(
        event_handlers: Vec<Box<dyn EventHandler + Sync + Send>>,
        event_stream: Pin<Box<dyn Stream<Item = Event> + Sync + Send>>,
    ) -> Self {
        Self {
            event_handlers,
            event_stream,
        }
    }

    pub async fn run(&mut self) -> Result<(), crate::error::Error> {
        let mut num_sleeps = 0;
        loop {
            if let Some(event) = self.event_stream.next().await {
                let futures = self
                    .event_handlers
                    .iter()
                    .map(|eh| async { eh.handle(&event).await });
                future::try_join_all(futures).await?;
            }
        }
        Ok(())
    }
}
