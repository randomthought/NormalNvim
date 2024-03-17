use crate::event::model::Signal;

#[derive(thiserror::Error, Debug)]
pub enum SignalError {
    #[error(transparent)]
    // TODO: the below is not probably not needed, model all errors
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("exceed max portfolio open trades")]
    ExceededMaxOpenTrades,
    #[error("exceed max portfolio loss error")]
    ExceededMaxPortfolioLoss,
    #[error("exceed max portfolio loss risk")]
    ExceededMaxPortfolioRisk,
    #[error("signal exceeds max portfolio open trades")]
    SignalExceedsMaxRiskPerTrade(Signal),
}
