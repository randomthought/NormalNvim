use std::sync::mpsc::{self, Receiver, Sender};

use domain::models::event::Event;
use domain::{
    engine::Engine,
    event::{channel_pipe::ChannelPipe, event::EventHandler},
};

#[tokio::main]
async fn main() {
    let pipe: ChannelPipe<'_> = ChannelPipe::default();
    let handlers: Vec<Box<dyn EventHandler>> = vec![];
    let mut algo_engine: Engine<'_> = Engine::new(handlers, Box::new(pipe));

    tokio::spawn(async move {
        // let algo_engine = Engine::new(handlers, &pipe);
        algo_engine.runner().await;
    });
}
