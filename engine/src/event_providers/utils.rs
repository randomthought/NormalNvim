use std::{pin::Pin, sync::Arc};

use domain::event::event::EventProducer;
use futures_util::{Stream, StreamExt};

use super::provider::Parser;
use anyhow::Result;

pub struct EventStream {
    event_producer: Arc<dyn EventProducer>,
    data_stream: Pin<Box<dyn Stream<Item = Result<String>>>>,
    parser: Box<dyn Parser>,
}

impl EventStream {
    pub fn new(
        event_producer: Arc<dyn EventProducer>,
        data_stream: Pin<Box<dyn Stream<Item = Result<String>>>>,
        parser: Box<dyn Parser>,
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
            let events = self.parser.parse(&raw_data)?;
            for e in events {
                self.event_producer.produce(e).await?;
            }
        }

        Ok(())
    }
}
