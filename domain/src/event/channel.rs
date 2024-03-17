use super::{event::EventProducer, model::Event};
use async_trait::async_trait;
use crossbeam::channel::{unbounded, Receiver, Sender};
use futures_util::Stream;
use std::sync::Arc;

#[derive(Clone)]
pub struct Channel {
    receiver: Arc<Receiver<Event>>,
    sender: Arc<Sender<Event>>,
}

impl Channel {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender: Arc::new(sender),
            receiver: Arc::new(receiver),
        }
    }
}

#[async_trait]
impl EventProducer for Channel {
    async fn produce(&self, event: Event) -> Result<(), crate::error::Error> {
        let sender = self.sender.clone();
        sender
            .send(event.clone())
            .map_err(|e| crate::error::Error::Any(e.into()))?;
        Ok(())
    }
}

impl Stream for Channel {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.receiver.recv() {
            Ok(data) => return std::task::Poll::Ready(Some(data)),
            _ => std::task::Poll::Ready(None),
        }
    }
}
