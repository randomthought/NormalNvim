use crate::data_providers::polygon::{models::Aggregates, utils};
use anyhow::Result;
use domain::{engine::Parser, models::price::PriceHistory};

pub struct PolygonParser;

impl PolygonParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for PolygonParser {
    fn parse(&self, data: &str) -> Result<Box<dyn Iterator<Item = PriceHistory>>> {
        let deserialized: Vec<Aggregates> = serde_json::from_str(data)
            .expect(format!("Unable to deserialize data: {}", data).as_str());

        let s: Result<Vec<_>, _> = deserialized
            .into_iter()
            .map(|ag| utils::to_price_history(&ag))
            .collect();

        let results = s?.into_iter();

        Ok(Box::new(results))
    }
}
