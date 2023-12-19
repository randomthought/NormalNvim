use anyhow::Result;
use domain::event::model::Event;

pub trait Parser {
    fn parse(&mut self, data: &str) -> Result<Vec<Event>>;
}
