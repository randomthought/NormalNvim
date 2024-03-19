use actix::{Message, Recipient};
use domain::{
    event::model::Event,
    strategy::model::{algo_event::AlgoEvent, signal::Signal},
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct EventMessage(pub Event);

#[derive(Message)]
#[rtype(result = "()")]
// TODO: should return an error
pub struct SignalMessage(pub Signal);

#[derive(Message)]
#[rtype(result = "()")]
// TODO: should return an error
pub struct AlgoEventMessage(pub AlgoEvent);

#[derive(Message)]
#[rtype(result = "()")]
// TODO: should return an error
pub struct AddSignalSubscribers(pub Recipient<SignalMessage>);
