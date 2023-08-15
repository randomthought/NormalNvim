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

#[derive(Clone)]
pub struct ChannelPipe {
    sender: Arc<Mutex<Sender<Event>>>,
    reciever: Arc<Mutex<Receiver<Event>>>,
}

impl Default for ChannelPipe {
    fn default() -> Self {
        let (sx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();
        let sm = Mutex::new(sx);
        let asm = Arc::new(sm);
        let rm = Mutex::new(rx);
        let arm = Arc::new(rm);
        Self {
            sender: asm,
            reciever: arm,
        }
    }
}

#[async_trait]
impl Pipe for ChannelPipe {
    async fn send(&self, event: Event) -> Result<(), io::Error> {
        let sender = self.sender.lock().await;
        match sender.send(event) {
            Ok(r) => Ok(r),
            Err(err) => Err(io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }

    async fn recieve(&self) -> Result<Option<Event>, io::Error> {
        let reciever = self.reciever.lock().await;
        match reciever.recv() {
            Ok(event) => Ok(Some(event)),
            Err(err) => Err(io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }
}

#[async_trait]
impl EventProducer for ChannelPipe {
    async fn produce(&self, event: Event) -> Result<(), io::Error> {
        self.send(event).await
    }
}
