use crate::models::event::Event;
use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait EventHandler: Sync + Send {
    async fn handle(&self, event: &Event) -> Result<(), io::Error>;
}

#[async_trait]
pub trait EventProducer: Sync + Send {
    async fn produce(&self, event: &Event) -> Result<(), io::Error>;
}

#[async_trait]
pub trait Pipe: Sync + Send {
    async fn send(&self, event: Event) -> Result<(), io::Error>;
    async fn recieve(&self) -> Result<Option<Event>, io::Error>;
}
