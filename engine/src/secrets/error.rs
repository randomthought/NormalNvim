#[derive(thiserror::Error, Debug)]
pub enum SecretError {
    #[error("secrete `{0}` not found")]
    SecretNotFound(String),
    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
}
