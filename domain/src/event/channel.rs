use super::{event::EventProducer, model::Event};
use async_trait::async_trait;
use tokio::sync::mpsc::{self, Receiver, Sender};
// use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{pin::Pin, sync::Arc};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};

#[derive(Clone)]
pub struct ChannelProducer {
    sender: Sender<Event>,
}

impl ChannelProducer {
    pub fn new(sender: Sender<Event>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl EventProducer for ChannelProducer {
    async fn produce(&self, event: Event) -> Result<(), crate::error::Error> {
        // let sender = self.sender.clone();
        // sender
        //     .send(event.clone())
        //     .await
        //     .map_err(|e| crate::error::Error::Any(e.into()))?;

        Ok(())
    }
}
