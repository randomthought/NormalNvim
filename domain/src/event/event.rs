use crate::models::{event::Event, price::PriceHistory};
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
