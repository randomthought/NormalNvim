use crate::event::{event::EventHandler, model::Event};
use anyhow::Result;
use futures_util::future;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct Runner {
    event_handlers: Vec<Box<dyn EventHandler + Sync + Send>>,
    event_stream: Pin<Box<dyn Stream<Item = Event> + Sync + Send>>,
    exit_signal: Arc<AtomicBool>,
}

impl Runner {
    pub fn new(
        event_handlers: Vec<Box<dyn EventHandler + Sync + Send>>,
        event_stream: Pin<Box<dyn Stream<Item = Event> + Sync + Send>>,
        exit_signal: Arc<AtomicBool>,
    ) -> Self {
        Self {
            event_handlers,
            event_stream,
            exit_signal,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut num_sleeps = 0;
        loop {
            if self.exit_signal.load(Ordering::Relaxed) && num_sleeps >= 4 {
                return Ok(());
            }

            if let Some(event) = self.event_stream.next().await {
                num_sleeps = 0;

                let futures = self
                    .event_handlers
                    .iter()
                    .map(|eh| async { eh.handle(&event).await });
                future::try_join_all(futures).await?;
            } else {
                sleep(Duration::from_millis(500)).await;
                num_sleeps += 1;
            }
        }
    }
}
