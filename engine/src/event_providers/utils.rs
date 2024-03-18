use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use domain::event::event::EventProducer;
use futures_util::{Stream, StreamExt};
use tokio::time::sleep;

use super::provider::Parser;
use color_eyre::eyre::Result;

pub struct EventStream {
    event_producer: Arc<dyn EventProducer + Sync + Send>,
    data_stream: Pin<Box<dyn Stream<Item = Result<String>> + Sync + Send>>,
    parser: Arc<dyn Parser + Sync + Send>,
}

impl EventStream {
    pub fn new(
        event_producer: Arc<dyn EventProducer + Sync + Send>,
        data_stream: Pin<Box<dyn Stream<Item = Result<String>> + Sync + Send>>,
        parser: Arc<dyn Parser + Sync + Send>,
    ) -> Self {
        Self {
            event_producer,
            data_stream,
            parser,
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

        Ok(())
    }
}
