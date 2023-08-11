use std::sync::mpsc::{self, Receiver, Sender};

use domain::{engine::Engine, event::EventHandler, models::event::Event};
use engine::ChannelPipe;

#[tokio::main]
async fn main() {
    let (sx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();
    let pipe = ChannelPipe::new(sx, rx);

    let handlers: Vec<&dyn EventHandler> = vec![];

    let t1 = tokio::spawn(async move {
        let mut algo_engine = Engine::new(handlers, &pipe);
        algo_engine.runner().await;
    });
}
