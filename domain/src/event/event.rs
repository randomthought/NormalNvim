use crate::models::event::Event;
use async_trait::async_trait;
use std::io;

#[async_trait]
// TODO: this is unsafe, make sure your implmentations are actually Sync + Send
pub(crate) trait EventHandler<'a>: Send + Sync {
    async fn handle(&self, event: Event<'a>) -> Result<(), io::Error>;
}

#[async_trait]
// TODO: this is unsafe, make sure your implmentations are actually Sync + Send
pub(crate) trait EventProducer<'a>: Send + Send {
    async fn produce(&self, event: Event<'a>) -> Result<(), io::Error>;
}

#[async_trait]
pub trait Pipe<'a> {
    async fn send(&self, event: Event<'a>) -> Result<(), io::Error>;
    async fn recieve(&self) -> Result<Option<Event<'a>>, io::Error>;
}
