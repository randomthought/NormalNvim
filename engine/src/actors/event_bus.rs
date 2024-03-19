use actix::{dev::SendError, Actor, Context, Recipient};
use domain::{event::model::DataEvent, strategy::model::algo_event::AlgoEvent};

use super::models::AlgoEventMessage;

#[derive(Default, Clone)]
pub struct EventBus {
    pub subscribers: Vec<Recipient<AlgoEventMessage>>,
}

impl EventBus {
    /// Send event to all subscribers
    pub fn notify(&self, event: DataEvent) -> Result<(), SendError<AlgoEventMessage>> {
        for subscriber in &self.subscribers {
            let data_event = event.clone();
            let algo_event = AlgoEvent::DataEvent(data_event);
            subscriber.try_send(AlgoEventMessage(algo_event))?;
        }

        Ok(())
    }
}

impl Actor for EventBus {
    type Context = Context<Self>;
}
