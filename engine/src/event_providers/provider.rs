use anyhow::Result;
use domain::event::model::Event;

pub trait Parser: Sync + Send {
    fn parse(&mut self, data: &str) -> Result<Box<dyn Iterator<Item = Event> + Sync + Send>>;
}
