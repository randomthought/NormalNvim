use std::collections::VecDeque;
use std::time::Duration;

use async_trait::async_trait;
use domain::event::model::DataEvent;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::event_providers::provider::Parser;
use crate::event_providers::provider::ParserError;

use super::{models::Aggregates, utils};

pub struct PolygonParser {
    event_queue: Mutex<VecDeque<DataEvent>>,
}

impl PolygonParser {
    pub fn new() -> Self {
        Self {
            event_queue: Mutex::new(VecDeque::new()),
        }
    }
}

#[async_trait]
impl Parser for PolygonParser {
    async fn parse(&self, data: &str) -> Result<Option<DataEvent>, ParserError> {
        if data.is_empty() {
            return Ok(None);
        }

        let deserialized: Vec<Aggregates> =
            serde_json::from_str(data).map_err(|e| ParserError::UnableToParseData(data.into()))?;

        let results: Result<Vec<_>, _> = deserialized
            .into_iter()
            .map(|ag| utils::to_price_history(&ag).map(|cnd| DataEvent::Candle(cnd)))
            .collect();

        let events = results.map_err(|e| ParserError::OtherError(e.into()))?;

        let mut event_queue = self.event_queue.lock().await;
        for event in events {
            event_queue.push_back(event);
        }

        if let Some(event) = event_queue.pop_front() {
            // sleep(Duration::from_millis(1)).await;

            return Ok(Some(event));
        }

        Err(ParserError::UnableToParseData(data.into()))
    }
}
