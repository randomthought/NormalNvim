use async_trait::async_trait;
use futures_util::lock::Mutex;
use std::{
    io::{self, ErrorKind},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
};

use crate::models::event::Event;

use super::event::{EventProducer, Pipe};

pub struct ChannelPipe<'a> {
    sender: Arc<Mutex<Sender<Event<'a>>>>,
    reciever: Arc<Mutex<Receiver<Event<'a>>>>,
}

impl<'a> Default for ChannelPipe<'a> {
    fn default() -> Self {
        let (sx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();
        let sm = Mutex::new(sx);
        let rm = Mutex::new(rx);
        Self {
            reciever: Arc::new(rm),
            sender: Arc::new(sm),
        }
    }
}

#[async_trait]
impl<'a> Pipe<'a> for ChannelPipe<'a> {
    async fn send(&self, event: Event<'a>) -> Result<(), io::Error> {
        let sender = self.sender.lock().await;
        match sender.send(event) {
            Ok(r) => Ok(r),
            Err(err) => Err(io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }

    async fn recieve(&self) -> Result<Option<Event<'a>>, io::Error> {
        let reciever = self.reciever.lock().await;
        match reciever.recv() {
            Ok(event) => Ok(Some(event)),
            Err(err) => Err(io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }
}

#[async_trait]
impl<'a> EventProducer<'a> for ChannelPipe<'a> {
    async fn produce(&self, event: Event<'a>) -> Result<(), io::Error> {
        self.send(event).await
    }
}
