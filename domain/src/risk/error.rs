#[derive(thiserror::Error, Debug)]
pub enum RiskError {
    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("exceeded maximum number of trades")]
    ExceededMaxOpenPortfolioTrades,
    #[error("trading is set to halted")]
    TradingHalted,
    #[error("signal type is not supported")]
    UnsupportedSignalType,
}
