use super::{event::EventProducer, model::Event};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};

pub struct EventChannel<'a> {
    receiver: Arc<Mutex<Receiver<&'a Event>>>,
    sender: Arc<Mutex<Sender<&'a Event>>>,
}

impl<'a> EventChannel<'a> {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let s = Arc::new(Mutex::new(receiver));
        Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[async_trait]
impl<'a> EventProducer for EventChannel<'a> {
    async fn produce(&self, event: &Event) -> Result<()> {
        let sender = self.sender.lock().unwrap();
        sender.send(event)?;
        Ok(())
    }
}

impl<'a> Stream for EventChannel<'a> {
    type Item = &'a Event;

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
