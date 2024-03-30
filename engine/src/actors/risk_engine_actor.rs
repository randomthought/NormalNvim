use std::{collections::HashMap, time::Instant};

use actix::{dev::ContextFutureSpawner, Actor, Addr, Context, Handler, WrapFuture};
use derive_builder::Builder;
use domain::{
    models::orders::order_result::OrderResult,
    risk::{error::RiskError, risk_engine::RiskEngine},
    strategy::{algorithm::StrategyId, model::algo_event::AlgoEvent},
};
use futures_util::Future;
use opentelemetry::{
    metrics::{Counter, Histogram, ObservableGauge},
    KeyValue,
};

use super::{
    algo_actor::AlgoActor,
    models::{AlgoEventMessage, SignalMessage},
};

#[derive(Builder, Clone)]
#[builder(setter(prefix = "with"))]
pub struct RiskEngineActor {
    pub risk_engine: RiskEngine,
    #[builder(private)]
    pub subscribers: HashMap<StrategyId, Addr<AlgoActor>>,
    pub risk_engine_error_counter: Counter<u64>,
    pub risk_engine_order_result_counter: Counter<u64>,
    pub risk_engine_order_result_gauge: ObservableGauge<u64>,
    pub risk_engine_process_signal_histogram: Histogram<f64>,
}

impl RiskEngineActor {
    pub fn builder() -> RiskEngineActorBuilder {
        RiskEngineActorBuilder::default()
    }

    async fn instrument(
        &self,
        strategy_id: StrategyId,
        future: impl Future<Output = Result<Vec<OrderResult>, RiskError>>,
    ) -> Result<Vec<OrderResult>, RiskError> {
        let default_attrs = &[KeyValue::new("strategy_id", strategy_id)];

        let start_time = Instant::now();
        let result = future.await;
        match result.as_ref() {
            Ok(ods) => {
                for or in ods.iter() {
                    self.risk_engine_order_result_counter.add(
                        1,
                        &[
                            KeyValue::new("strategy_id", strategy_id),
                            KeyValue::new("order_result", format!("{}", or.as_ref())),
                        ],
                    );
                }
            }
            Err(e) => {
                self.risk_engine_error_counter
                    .add(1, &[KeyValue::new("error", format!("{}", e.as_ref()))]);
            }
        }

        let elapsed = start_time.elapsed().as_millis() as f64;

        self.risk_engine_process_signal_histogram
            .record(elapsed, default_attrs);

        result
    }
}

impl RiskEngineActorBuilder {
    pub fn add_subscriber(
        &mut self,
        strategy_id: StrategyId,
        address: Addr<AlgoActor>,
    ) -> &mut Self {
        if let Some(subscribers) = self.subscribers.as_mut() {
            subscribers.insert(strategy_id, address);
            return self;
        }

        let mut subscribers = HashMap::new();
        subscribers.insert(strategy_id, address);

        self.subscribers = Some(subscribers);
        self
    }
}

impl Actor for RiskEngineActor {
    type Context = Context<Self>;
}

impl Handler<SignalMessage> for RiskEngineActor {
    type Result = ();

    fn handle(&mut self, msg: SignalMessage, ctx: &mut Self::Context) -> Self::Result {
        let signal = msg.0;
        let self_ = self.clone();
        async move {
            let future = self_.risk_engine.process_signal(&signal);
            let result = self_.instrument(&signal.strategy_id(), future).await;
            let Ok(order_results) = result.as_ref() else {
                return;
            };
            for or in order_results {
                if let Some(subscriber) = self_.subscribers.get(or.startegy_id()) {
                    let event_msg = AlgoEventMessage(AlgoEvent::OrderResult(or.clone()));
                    subscriber.send(event_msg).await;
                }
            }
        }
        .into_actor(self)
        .wait(ctx)
    }
}
