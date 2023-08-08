use crate::models::event::Event;
use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait EventHandler {
    async fn handle(&self, event: &Event) -> Result<(), io::Error>;
}
