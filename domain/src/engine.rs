use crate::{
    event::{
        event::EventHandler,
        model::{Event, Market},
    },
    models::price::PriceHistory,
};
use anyhow::Result;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

pub trait Parser {
    fn parse(&mut self, data: &str) -> Result<Box<dyn Iterator<Item = PriceHistory>>>;
}

pub struct Engine {
    event_handlers: Vec<Box<dyn EventHandler>>,
    parser: Box<dyn Parser>,
    data_stream: Pin<Box<dyn Stream<Item = Result<String>>>>,
}

impl Engine {
    pub fn new(
        event_handlers: Vec<Box<dyn EventHandler>>,
        parser: Box<dyn Parser>,
        data_stream: Pin<Box<dyn Stream<Item = Result<String>>>>,
    ) -> Self {
        Self {
            event_handlers,
            parser,
            data_stream,
        }
    }

    pub async fn runner(&mut self) -> Result<()> {
        loop {
            match self.data_stream.next().await {
                Some(Ok(data)) => self.process_data(&data).await?,
                Some(Err(err)) => return Err(err),
                _ => (),
            }
        }
    }

    async fn process_data(&mut self, data: &str) -> Result<()> {
        let events = self
            .parser
            .parse(data)?
            .map(|ph| Event::Market(Market::DataEvent(ph)));

        for event in events {
            self.handle_event(&event).await?;
        }

        Ok(())
    }

    async fn handle_event(&self, event: &Event) -> Result<()> {
        for eh in &self.event_handlers {
            eh.handle(event).await?;
        }

        Ok(())
    }
}
