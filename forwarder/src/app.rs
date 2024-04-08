use std::{
    env,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use actix_web::{
    web::{self, Bytes},
    App, HttpResponse, HttpServer, Responder,
};
use color_eyre::eyre::Result;
use data_providers::{
    file,
    market::polygon::{self, parser::PolygonParser},
    utils,
};
use futures_util::StreamExt;
use models::event::DataEvent;
use tokio::sync::broadcast;

struct AppState {
    sender: broadcast::Sender<DataEvent>,
}

async fn market_aggregate_stream(state: web::Data<AppState>) -> impl Responder {
    let receiver = state.sender.subscribe();
    let stream = futures_util::stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.ok().map(|DataEvent::PriceBar(v)| {
            let results = serde_json::to_vec(&v);

            (results.map(Bytes::from), receiver)
        })
    })
    .boxed_local();

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream)
}

async fn stream_market_data(
    sender: broadcast::Sender<DataEvent>,
    shutdown_signal: Arc<AtomicBool>,
) -> eyre::Result<()> {
    let file = env::var("FILE")?;
    let path = Path::new(&file);
    let buff_size = 4096usize;

    let parser = Arc::new(PolygonParser::new());
    let raw_data_stream = file::utils::create_stream(path, buff_size)?;
    let mut data_stream = utils::parse_stream(raw_data_stream, parser.clone());

    while let Some(dr) = data_stream.next().await {
        if shutdown_signal.load(Ordering::SeqCst) {
            break;
        }

        if let Some(data_event) = dr? {
            let _ = sender.send(data_event);
        }
    }

    Ok(())
}

async fn run_polygon_stream(
    sender: broadcast::Sender<DataEvent>,
    shutdown_signal: Arc<AtomicBool>,
) -> eyre::Result<()> {
    let api_key = env::var("API_KEY")?;
    let subscription = "A.*";

    let raw_data_stream = polygon::stream_client::create_stream(&api_key, &subscription)?;
    let parser = Arc::new(PolygonParser::new());
    let mut data_stream = utils::parse_stream(raw_data_stream, parser.clone());

    while let Some(dr) = data_stream.next().await {
        if shutdown_signal.load(Ordering::SeqCst) {
            break;
        }

        if let Some(data_event) = dr? {
            let _ = sender.send(data_event);
        }
    }

    Ok(())
}

pub async fn run_app() -> Result<()> {
    color_eyre::install()?;

    let (sender, _) = broadcast::channel(100); // Creating a broadcast channel
    let state = web::Data::new(AppState {
        sender: sender.clone(),
    });

    let shutdown_signal = Arc::new(AtomicBool::new(false));

    let runner = stream_market_data(sender, shutdown_signal.clone());
    // let runner = run_polygon_stream(sender, shutdown_signal.clone());

    let sever = HttpServer::new(move || {
        App::new()
            .app_data(state.clone()) // Add the shared state to the app
            .route("/market", web::get().to(market_aggregate_stream))
    })
    .bind("127.0.0.1:8081")?
    .run();

    tokio::spawn(sever);

    tokio::select! {
         _ = tokio::signal::ctrl_c() => {
            println!("Shutdown signal received, shutting down...");
            shutdown_signal.store(true, Ordering::SeqCst);
        },
        Err(e) = runner => {
            println!("Server error or shutdown");
            return Err(e);
        },
    }

    Ok(())
}
