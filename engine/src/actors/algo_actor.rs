use std::sync::Arc;

use actix::{dev::ContextFutureSpawner, Actor, Context, Handler, Recipient, WrapFuture};
use domain::strategy::{
    algorithm::Algorithm,
    model::{algo_event::AlgoEvent, signal::Signal},
};

use super::models::{AddSignalSubscribers, AlgoEventMessage, SignalMessage};

#[derive(Clone)]
pub struct AlgoActor {
    pub algorithm: Arc<dyn Algorithm>,
    pub subscribers: Vec<Recipient<SignalMessage>>,
}

impl AlgoActor {
    fn notify(&self, signal: Signal) {
        for subscriber in &self.subscribers {
            subscriber.do_send(SignalMessage(signal.clone()));
        }
    }

    pub fn add_subscriber(&mut self, subscriber: Recipient<SignalMessage>) {
        self.subscribers.push(subscriber)
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
        let subscribers = self.subscribers.clone();
        async move {
            if let Ok(Some(signal)) = algorithm.on_event(algo_event).await {
                for subscriber in subscribers {
                    let _ = subscriber.send(SignalMessage(signal.clone())).await;
                }
            }
        }
        .into_actor(self)
        .wait(ctx);
    }
}

impl Handler<AddSignalSubscribers> for AlgoActor {
    type Result = ();

    fn handle(&mut self, msg: AddSignalSubscribers, ctx: &mut Self::Context) -> Self::Result {
        self.subscribers.push(msg.0);
    }
}
