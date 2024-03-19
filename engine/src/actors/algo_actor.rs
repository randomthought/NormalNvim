use std::sync::Arc;

use actix::{dev::ContextFutureSpawner, Actor, Context, Handler, Recipient, WrapFuture};
use domain::strategy::{
    algorithm::Algorithm,
    model::{algo_event::AlgoEvent, signal::Signal},
};

use super::models::{AlgoEventMessage, SignalMessage};

#[derive(Clone)]
pub struct AlgoActor {
    pub algorithm: Arc<dyn Algorithm>,
    pub subscribers: Vec<Recipient<SignalMessage>>,
}

impl AlgoActor {
    fn notify(&mut self, signal: Signal) {
        for subscriber in &self.subscribers {
            subscriber.do_send(SignalMessage(signal.clone()));
        }
    }
}

impl Actor for AlgoActor {
    type Context = Context<Self>;
}

impl Handler<AlgoEventMessage> for AlgoActor {
    type Result = ();
    fn handle(&mut self, msg: AlgoEventMessage, ctx: &mut Self::Context) -> Self::Result {
        if let AlgoEvent::OrderResult(x) = msg.0.clone() {
            if x.startegy_id() != self.algorithm.strategy_id() {
                return;
            }
        }

        let algo_event = msg.0;
        let algorithm = self.algorithm.clone();
        async move {
            // TODO: think about how you would deal with errors and monitoring here. Imagine, you cannot close a position!
            algorithm.on_event(algo_event).await;
        }
        .into_actor(self)
        .wait(ctx);
    }
}
