use async_trait::async_trait;
use color_eyre::eyre::Result;

use super::model::Event;

#[async_trait]
pub trait EventProducer {
    async fn produce(&self, event: Event) -> Result<()>;
}

#[async_trait]
pub trait EventHandler {
    async fn handle(&self, event: &Event) -> Result<()>;
}
