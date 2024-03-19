use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    event::model::Signal,
    models::{
        order::{self, Side},
        price::Quote,
    },
    risk::config::RiskEngineConfig,
};

pub struct RiskUtils {
    risk_engine_config: RiskEngineConfig,
}
use color_eyre::eyre::{Context, Ok, Result};

impl RiskUtils {
    fn calulate_risk_quantity(
        &self,
        account_value: Decimal,
        qoute: &Quote,
        side: Side,
    ) -> Result<u64> {
        let obtain_price = match side {
            order::Side::Long => qoute.ask,
            order::Side::Short => qoute.bid,
        };

        // TODO: think about making risk engine values all decimals?
        let max_risk_per_trade = Decimal::from_f64(self.risk_engine_config.max_risk_per_trade)
            .context(format!(
                "unable to parse '{}' to decimal",
                self.risk_engine_config.max_risk_per_trade
            ))?;

        let max_trade_loss = account_value * max_risk_per_trade;

        let risk_amount = (obtain_price - signal.stop).abs();

        let quantity = (max_trade_loss / risk_amount).trunc();

        let result = u64::try_from(quantity)?;
        Ok(result)
    }
}
