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
    ExceededAlgoOpenTrades,
    #[error("exceed max algorithm loss")]
    ExceededAlgoMaxLoss,
    #[error("insufficient algoirthim account balance")]
    InsufficientAlgoAccountBalance,
    #[error("unable to find algorithm risk config {0}")]
    UnableToFindAlgoRiskConfig(StrategyId),
    #[error("exceeded max portfolio open trades")]
    ExceededAlgoRiskPerTrade(Signal),
    #[error("exceeded max portfolio ")]
    ExceededPortfolioRiskPerTrade,
    #[error("exceeded portfolio open trades")]
    ExceededPortfolioOpenTrades,
    #[error("exceeded portfolio risk")]
    ExceededPortfolioRisk,
    #[error("signal type is not supported")]
    UnsupportedSignalType(Signal),
}
