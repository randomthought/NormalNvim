use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use domain::event::event::EventProducer;
use futures_util::{Stream, StreamExt};

use super::provider::Parser;
use anyhow::Result;

pub struct EventStream {
    event_producer: Arc<dyn EventProducer + Sync + Send>,
    data_stream: Pin<Box<dyn Stream<Item = Result<String>> + Sync + Send>>,
    parser: Arc<dyn Parser + Sync + Send>,
    exit_signal: Arc<AtomicBool>,
}

impl EventStream {
    pub fn new(
        event_producer: Arc<dyn EventProducer + Sync + Send>,
        data_stream: Pin<Box<dyn Stream<Item = Result<String>> + Sync + Send>>,
        parser: Arc<dyn Parser + Sync + Send>,
        exit_signal: Arc<AtomicBool>,
    ) -> Self {
        Self {
            event_producer,
            data_stream,
            parser,
            exit_signal,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        while let Some(dr) = self.data_stream.next().await {
            let raw_data = dr?;
            let events = self.parser.parse(&raw_data).await?;
            for e in events {
                self.event_producer.produce(e).await?;
            }
        }

        self.exit_signal.store(true, Ordering::Relaxed);

        println!("finish processing stream");

        Ok(())
    }
}
