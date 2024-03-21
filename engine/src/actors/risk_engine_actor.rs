use std::collections::HashMap;

use actix::{dev::ContextFutureSpawner, Actor, Addr, Context, Handler, Recipient, WrapFuture};
use derive_builder::Builder;
use domain::{
    risk::risk_engine::{RiskEngine, RiskEngineBuilder},
    strategy::{algorithm::StrategyId, model::algo_event::AlgoEvent},
};

use super::{
    algo_actor::AlgoActor,
    models::{AlgoEventMessage, SignalMessage},
};

#[derive(Builder, Clone)]
pub struct RiskEngineActor {
    #[builder(setter(prefix = "with"))]
    pub risk_engine: RiskEngine,
    #[builder(private)]
    pub subscribers: HashMap<StrategyId, Addr<AlgoActor>>,
}

impl RiskEngineActor {
    pub fn builder() -> RiskEngineActorBuilder {
        RiskEngineActorBuilder::default()
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
        let subscribers = self.subscribers.clone();
        let risk_engine = self.risk_engine.clone();
        async move {
            let Ok(order_results) = risk_engine.process_signal(&signal).await else {
                return;
            };
            for or in order_results {
                if let Some(subscriber) = subscribers.get(or.startegy_id()) {
                    let event_msg = AlgoEventMessage(AlgoEvent::OrderResult(or.clone()));
                    subscriber.send(event_msg).await;
                }
            }
        }
        .into_actor(self)
        .wait(ctx)
    }
}
