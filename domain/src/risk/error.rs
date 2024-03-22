use crate::strategy::{algorithm::StrategyId, model::signal::Signal};

#[derive(thiserror::Error, Debug)]
pub enum RiskError {
    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("trading is set to halted")]
    TradingHalted,
    #[error("exceed max algorithm open trades")]
    ExceededAlgoMaxOpenTrades,
    #[error("exceed max algorithm loss")]
    ExceededAlgoMaxLoss,
    #[error("unable to find algorithm risk config {0}")]
    UnableToFindAlgoRiskConfig(StrategyId),
    #[error("exceeded max portfolio open trades")]
    ExceededAlgoMaxRiskPerTrade(Signal),
    #[error("exceeded max portfolio ")]
    ExceededPortfolioMaxRiskPerTrade,
    #[error("exceeded max portfolio open trades")]
    ExceededPortfolioMaxOpenTrades,
    #[error("signal type is not supported")]
    UnsupportedSignalType,
}
