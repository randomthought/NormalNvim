use async_trait::async_trait;
use color_eyre::eyre::Result;
use domain::event::model::Event;

#[async_trait]
pub trait Parser {
    async fn parse(&self, data: &str) -> Result<Vec<Event>>;
}
