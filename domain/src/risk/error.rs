use strum_macros::AsRefStr;
use strum_macros::VariantNames;

use crate::strategy::{algorithm::StrategyId, model::signal::Signal};

#[derive(thiserror::Error, Debug, AsRefStr, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum RiskError {
    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("trading is set to halted")]
    TradingHalted,
    #[error("instrument is already traded by strategy_id=`{0}`")]
    InstrumentTradedByAglorithm(StrategyId),
    #[error("exceed max algorithm open trades")]
    ExceededAlgoMaxOpenTrades,
    #[error("exceed max algorithm loss")]
    ExceededAlgoMaxLoss,
    #[error("insufficient algoirthim account balance")]
    InsufficientAlgoAccountBalance,
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
