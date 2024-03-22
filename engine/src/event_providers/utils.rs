use std::{pin::Pin, sync::Arc, time::Duration};

use domain::event::model::DataEvent;
use eyre::Ok;
use futures_util::{Stream, StreamExt};
use tokio::time::sleep;

use super::provider::Parser;

pub fn parse_stream(
    stream: Pin<Box<dyn Stream<Item = eyre::Result<String>> + Send>>,
    parser: Arc<dyn Parser + Sync + Send>,
) -> Pin<Box<dyn Stream<Item = eyre::Result<DataEvent>> + Send>> {
    let mapped = stream.then(move |raw_data| transform_data(raw_data, parser.clone()));
    Box::pin(mapped)
}

async fn transform_data(
    data_result: eyre::Result<String>,
    parser: Arc<dyn Parser + Sync + Send>,
) -> eyre::Result<DataEvent> {
    let data = data_result?;

    sleep(Duration::from_millis(1)).await;
    let data_event = parser.parse(&data).await?;
    Ok(data_event)
}
