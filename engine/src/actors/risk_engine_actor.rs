use actix::{dev::ContextFutureSpawner, Actor, Addr, Context, Handler, Recipient, WrapFuture};
use domain::{risk::risk_engine::RiskEngine, strategy::model::algo_event::AlgoEvent};

use super::{
    algo_actor::AlgoActor,
    models::{AlgoEventMessage, SignalMessage},
};

#[derive(Clone)]
pub struct RiskEngineActor {
    pub risk_engine: RiskEngine,
    pub subscribers: Vec<Addr<AlgoActor>>,
}

impl RiskEngineActor {}

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
                for subscriber in subscribers.iter() {
                    let event_msg = AlgoEventMessage(AlgoEvent::OrderResult(or.clone()));
                    subscriber.send(event_msg).await;
                }
            }
        }
        .into_actor(self)
        .wait(ctx)
    }
}
