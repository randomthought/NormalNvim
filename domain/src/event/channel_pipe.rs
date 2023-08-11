use async_trait::async_trait;
use std::{
    io::{self, ErrorKind},
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

use crate::models::event::Event;

use super::event::{EventProducer, Pipe};

pub struct ChannelPipe {
    sender: Mutex<Sender<Event>>,
    reciever: Mutex<Receiver<Event>>,
}

impl ChannelPipe {
    pub fn new(sender: Sender<Event>, reciever: Receiver<Event>) -> Self {
        let rm = Mutex::new(reciever);
        let sm = Mutex::new(sender);
        Self {
            reciever: rm,
            sender: sm,
        };
        todo!()
    }
}

#[async_trait]
impl Pipe for ChannelPipe {
    async fn send(&self, event: Event) -> Result<(), io::Error> {
        let sender = self.sender.lock().unwrap();
        let ec = event.clone();
        sender.send(ec);
        todo!()
    }

    async fn recieve(&self) -> Result<Option<Event>, io::Error> {
        let reciever = self.reciever.lock().unwrap();
        match reciever.recv() {
            Ok(event) => Ok(Some(event)),
            Err(err) => Err(io::Error::new(ErrorKind::Other, err.to_string())),
        }
    }
}

#[async_trait]
impl EventProducer for ChannelPipe {
    async fn produce(&self, event: &Event) -> Result<(), io::Error> {
        let ec = event.clone();
        self.send(ec).await
    }
}
