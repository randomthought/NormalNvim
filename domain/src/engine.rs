use crate::models::event::Event;
use futures_util::Stream;
use std::io;
use std::pin::Pin;

use futures_util::StreamExt;

use crate::event::EventHandler;

pub struct Engine<'a> {
    handlers: &'a Vec<&'a dyn EventHandler>,
    pipe: &'a mut Pin<&'a mut dyn Stream<Item = &'a Event>>,
}

impl<'a> Engine<'a> {
    async fn runner(&mut self) -> Result<(), io::Error> {
        // while let Some(x) = self.pipe.next().await {}
        while let Some(event) = self.pipe.next().await {
            // TODO: handle events conncrrently
            for &ele in self.handlers {
                ele.handle(event).await?;
            }
        }

        Ok(())
    }
}
