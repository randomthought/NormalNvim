use domain::{event::EventProducer, models::event::Event};
use std::io;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub struct Pipe<'a> {
    sender: &'a Sender<&'a Event>,
    reciever: &'a Receiver<&'a Event>,
}

impl<'a> Pipe<'a> {
    pub fn new(&self, sender: &'a Sender<&'a Event>, reciever: &'a Receiver<&'a Event>) -> Self {
        Self { sender, reciever }
    }
}

impl<'a> IntoIterator for Pipe<'a> {
    type Item = &'a Event;

    type IntoIter = std::sync::mpsc::Iter<'a, &'a Event>;

    fn into_iter(self) -> Self::IntoIter {
        self.reciever.iter()
    }
}
