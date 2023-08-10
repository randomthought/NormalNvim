use crate::models::event::Event;
use futures_util::future;
use std::io;

use crate::event::EventHandler;

pub struct Engine<'a> {
    handlers: &'a Vec<&'a dyn EventHandler>,
    iter: &'a mut dyn Iterator<Item = &'a Event>,
}

impl<'a> Engine<'a> {
    async fn runner(&mut self) -> Result<(), io::Error> {
        while let Some(event) = self.iter.next() {
            let futures: Vec<_> = self
                .handlers
                .iter()
                .map(|algo| async move { algo.handle(event).await })
                .collect();

            let _ = future::try_join_all(futures).await;
        }

        Ok(())
    }
}
