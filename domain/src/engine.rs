use futures_util::future;
use std::io;

use crate::event::event::{EventHandler, Pipe};

pub struct Engine<'a> {
    handlers: Vec<Box<dyn EventHandler<'a>>>,
    pipe: Box<dyn Pipe<'a> + Send + Sync>,
}

impl<'a> Engine<'a> {
    pub fn new(
        handlers: Vec<Box<dyn EventHandler<'a>>>,
        pipe: Box<dyn Pipe<'a> + Send + Sync>,
    ) -> Self {
        Self { handlers, pipe }
    }

    pub async fn runner(&mut self) -> Result<(), io::Error> {
        while let Some(event) = self.pipe.recieve().await? {
            let futures: Vec<_> = self
                .handlers
                .iter()
                .map(|algo| async move { algo.handle(event).await })
                .collect();

            future::try_join_all(futures).await?;
        }

        Ok(())
    }
}
