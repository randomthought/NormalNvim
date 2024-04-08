#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    // a central IO wrapper
    #[error(transparent)]
    IO(#[from] std::io::Error),
    // will be used with `.map_err(Box::from)?;`
    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
