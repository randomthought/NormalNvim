use async_trait::async_trait;

use super::model::Event;

#[async_trait]
pub trait EventProducer {
    async fn produce(&self, event: Event) -> Result<(), crate::error::Error>;
}

#[async_trait]
pub trait EventHandler {
    async fn handle(&self, event: &Event) -> Result<(), crate::error::Error>;
}
