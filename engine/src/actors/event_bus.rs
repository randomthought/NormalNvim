use actix::{dev::SendError, Actor, Context, Recipient};
use domain::strategy::model::algo_event::AlgoEvent;

use super::models::AlgoEventMessage;

#[derive(Default, Clone)]
pub struct EventBus {
    pub subscribers: Vec<Recipient<AlgoEventMessage>>,
}

impl EventBus {
    /// Send event to all subscribers
    pub fn notify(&self, event: AlgoEvent) -> Result<(), SendError<AlgoEventMessage>> {
        for subscriber in &self.subscribers {
            subscriber.try_send(AlgoEventMessage(event.clone()))?;
        }

        Ok(())
    }
}

impl Actor for EventBus {
    type Context = Context<Self>;
}
