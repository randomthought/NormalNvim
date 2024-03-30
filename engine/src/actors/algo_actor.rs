use std::sync::Arc;

use actix::{dev::ContextFutureSpawner, Actor, Context, Handler, Recipient, WrapFuture};
use derive_builder::Builder;
use domain::strategy::algorithm::Algorithm;

use super::models::{AddSignalSubscribers, AlgoEventMessage, SignalMessage};

#[derive(Builder, Clone)]
pub struct AlgoActor {
    #[builder(public, setter(prefix = "with"))]
    algorithm: Arc<dyn Algorithm + Send + Send>,
    #[builder(public, default, setter(each = "add_subscriber"))]
    subscribers: Vec<Recipient<SignalMessage>>,
}

impl AlgoActor {
    pub fn builder() -> AlgoActorBuilder {
        AlgoActorBuilder::default()
    }
}

impl Actor for AlgoActor {
    type Context = Context<Self>;
}

impl Handler<AlgoEventMessage> for AlgoActor {
    type Result = ();
    fn handle(&mut self, msg: AlgoEventMessage, ctx: &mut Self::Context) -> Self::Result {
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
