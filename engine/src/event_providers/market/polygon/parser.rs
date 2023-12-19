use anyhow::Result;
use domain::event::model::{Event, Market};

use crate::event_providers::provider::Parser;

use super::{models::Aggregates, utils};

pub struct PolygonParser;

impl PolygonParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for PolygonParser {
    fn parse(&mut self, data: &str) -> Result<Vec<Event>> {
        if data.is_empty() {
            return Ok(vec![]);
        }

        println!("{data}");
        let deserialized: Vec<Aggregates> = serde_json::from_str(data)
            .expect(format!("Unable to deserialize data: {}", data).as_str());

        // TODO: can you lazily do this?
        let results: Result<Vec<_>, _> = deserialized
            .into_iter()
            .map(|ag| utils::to_price_history(&ag).map(|ph| Event::Market(Market::DataEvent(ph))))
            .collect();

        Ok(results?)
    }
}
