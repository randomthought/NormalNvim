#[derive(thiserror::Error, Debug)]
pub enum SecretError {
    #[error("{0}")]
    SecretNotFound(String),
    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
}
