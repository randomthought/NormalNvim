use std::{f64, sync::Arc, time::Instant, u64};

use async_trait::async_trait;
use derive_builder::Builder;
use domain::{
    models::orders::{pending_order::PendingOrder, security_position::SecurityPosition},
    strategy::{algorithm::StrategyId, portfolio::StrategyPortfolio},
};
use opentelemetry::{
    metrics::{Counter, Histogram, ObservableGauge},
    KeyValue,
};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

#[derive(Builder, Clone)]
#[builder(setter(prefix = "with"))]
pub struct StrategyPortfolioTelemtry {
    strategy_portfolio: Arc<dyn StrategyPortfolio + Send + Sync>,
    pub security_positions_gauge: ObservableGauge<u64>,
    pub security_positions_counter: Counter<u64>,
    pub get_security_positions_histogram: Histogram<f64>,
    pub get_security_positions_error_counter: Counter<u64>,
    pub profit_gauge: ObservableGauge<f64>,
    pub get_profit_histogram: Histogram<f64>,
    pub get_profit_error_counter: Counter<u64>,
    pub pending_orders_gauge: ObservableGauge<u64>,
    pub get_pending_histogram: Histogram<f64>,
    pub get_pending_error_counter: Counter<u64>,
}

impl StrategyPortfolioTelemtry {
    pub fn builder() -> StrategyPortfolioTelemtryBuilder {
        StrategyPortfolioTelemtryBuilder::default()
    }
}

#[async_trait]
impl StrategyPortfolio for StrategyPortfolioTelemtry {
    async fn get_profit(&self, strategy_id: StrategyId) -> Result<Decimal, domain::error::Error> {
        let default_attrs = &[KeyValue::new("strategy_id", strategy_id)];

        let start_time = Instant::now();
        let result = self.strategy_portfolio.get_profit(strategy_id).await;

        if let Ok(v) = result.as_ref() {
            let elapsed = start_time.elapsed().as_millis() as f64;
            self.get_profit_histogram.record(elapsed, default_attrs);

            let profit = Decimal::to_f64(v).ok_or(domain::error::Error::Message(format!(
                "error recording metric: unable to convert `{:?}` to f64",
                v
            )))?;

            self.profit_gauge.observe(profit, default_attrs);
        } else {
            self.get_profit_error_counter.add(1, default_attrs);
        }

        result
    }
    async fn get_security_positions(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<SecurityPosition>, domain::error::Error> {
        let default_attrs = &[KeyValue::new("strategy_id", strategy_id)];

        let start_time = Instant::now();
        let result = self
            .strategy_portfolio
            .get_security_positions(strategy_id)
            .await;

        if let Ok(v) = result.as_ref() {
            let elapsed = start_time.elapsed().as_millis() as f64;
            self.get_security_positions_histogram
                .record(elapsed, default_attrs);

            let holdings_count: u64 = v.len() as u64;

            self.security_positions_gauge
                .observe(holdings_count, default_attrs);
        } else {
            self.get_security_positions_error_counter
                .add(1, default_attrs);
        }

        result
    }

    async fn get_pending(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<PendingOrder>, domain::error::Error> {
        let default_attrs = &[KeyValue::new("strategy_id", strategy_id)];

        let start_time = Instant::now();
        let result = self.strategy_portfolio.get_pending(strategy_id).await;

        if let Ok(v) = result.as_ref() {
            let elapsed = start_time.elapsed().as_millis() as f64;
            self.get_pending_histogram.record(elapsed, default_attrs);

            let pending_count: u64 = v.len() as u64;

            self.pending_orders_gauge
                .observe(pending_count, default_attrs);
        } else {
            self.get_pending_error_counter.add(1, default_attrs);
        }

        result
    }
}
