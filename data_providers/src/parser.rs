use async_trait::async_trait;
use models::event::DataEvent;

#[derive(thiserror::Error, Debug)]
pub enum ParserError {
    #[error("enable to parse raw data `{0}`")]
    UnableToParseData(String),
    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[async_trait]
pub trait Parser {
    async fn parse(&self, data: &str) -> Result<Option<DataEvent>, ParserError>;
}
