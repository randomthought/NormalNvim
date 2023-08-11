use crate::event::EventHandler;
use crate::event::Pipe;
use futures_util::future;
use std::io;

pub struct Engine<'a> {
    handlers: &'a Vec<&'a dyn EventHandler>,
    pipe: &'a dyn Pipe,
}

impl<'a> Engine<'a> {
    pub fn new(handlers: &'a Vec<&'a dyn EventHandler>, pipe: &'a dyn Pipe) -> Self {
        Self { handlers, pipe }
    }

    pub async fn runner(&mut self) -> Result<(), io::Error> {
        // let c = self.pipe.recieve().await?

        while let Some(event) = self.pipe.recieve().await? {
            // TODO: Why do you need to clone here?
            let ec = event.clone();
            let futures: Vec<_> = self
                .handlers
                .iter()
                .map(|algo| async move { algo.handle(&ec).await })
                .collect();

            future::try_join_all(futures).await?;
        }

        Ok(())
    }
}
