use async_trait::async_trait;
use domain::event::model::DataEvent;

#[derive(thiserror::Error, Debug)]
pub enum ParserError {
    #[error("{0}")]
    UnableToParseData(String),
    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[async_trait]
pub trait Parser {
    async fn parse(&self, data: &str) -> Result<DataEvent, ParserError>;
}
