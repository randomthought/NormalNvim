#[derive(thiserror::Error, Debug)]
pub enum RiskError {
    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}
