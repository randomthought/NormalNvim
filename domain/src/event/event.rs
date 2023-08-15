use crate::models::event::Event;
use async_trait::async_trait;
use std::io;

#[async_trait]
// TODO: this is unsafe, make sure your implmentations are actually Sync + Send
pub(crate) trait EventHandler: Send + Sync {
    async fn handle(&self, event: Event) -> Result<(), io::Error>;
}

#[async_trait]
// TODO: this is unsafe, make sure your implmentations are actually Sync + Send
pub(crate) trait EventProducer: Send + Send {
    async fn produce(&self, event: Event) -> Result<(), io::Error>;
}

#[async_trait]
pub trait Pipe {
    async fn send(&self, event: Event) -> Result<(), io::Error>;
    async fn recieve(&self) -> Result<Option<Event>, io::Error>;
}
