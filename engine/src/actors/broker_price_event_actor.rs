use std::{collections::HashMap, sync::Arc};

use actix::{dev::ContextFutureSpawner, Actor, Addr, Context, Handler, WrapFuture};
use derive_builder::Builder;
use domain::broker::Broker;
use models::{
    event::DataEvent,
    strategy::{algo_event::AlgoEvent, common::StrategyId},
};

use super::{algo_actor::AlgoActor, models::AlgoEventMessage};

#[derive(Builder, Clone)]
#[builder(public, setter(prefix = "with"))]
pub struct BrokerPriceEventActor {
    in_memory_broker: Arc<Broker>,
    #[builder(private)]
    subscribers: HashMap<StrategyId, Addr<AlgoActor>>,
}

impl BrokerPriceEventActor {
    pub fn builder() -> BrokerPriceEventActorBuilder {
        BrokerPriceEventActorBuilder::default()
    }
}

impl BrokerPriceEventActorBuilder {
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

impl Actor for BrokerPriceEventActor {
    type Context = Context<Self>;
}

impl Handler<AlgoEventMessage> for BrokerPriceEventActor {
    type Result = ();
    fn handle(&mut self, msg: AlgoEventMessage, ctx: &mut Self::Context) -> Self::Result {
        let AlgoEvent::DataEvent(DataEvent::PriceBar(candle)) = msg.0 else {
            return;
        };
        let broker = self.in_memory_broker.clone();
        let subscribers = self.subscribers.clone();
        async move {
            let Ok(order_results) = broker.handle(&candle).await else {
                return;
            };

            for order_result in order_results {
                if let Some(subscriber) = subscribers.get(order_result.startegy_id()) {
                    let algo_event = AlgoEvent::OrderResult(order_result);
                    let _ = subscriber.send(AlgoEventMessage(algo_event)).await;
                }
            }
        }
        .into_actor(self)
        .wait(ctx);
    }
}
