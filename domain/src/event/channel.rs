use super::{event::EventProducer, model::Event};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};

pub struct EventChannel {
    receiver: Arc<Mutex<Receiver<Event>>>,
    sender: Arc<Mutex<Sender<Event>>>,
}

impl EventChannel {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[async_trait]
impl EventProducer for EventChannel {
    async fn produce(&self, event: Event) -> Result<()> {
        let sender = self.sender.lock().unwrap();
        sender.send(event.clone())?;
        Ok(())
    }
}

impl Stream for EventChannel {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // TODO: Look why could potentially fail here
        match self.receiver.lock().unwrap().recv() {
            Ok(data) => return std::task::Poll::Ready(Some(data)),
            _ => std::task::Poll::Ready(None),
        }
    }
}
