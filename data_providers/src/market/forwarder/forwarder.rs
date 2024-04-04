use bytes::Bytes;
use derive_builder::Builder;
use domain::event::model::DataEvent;
use domain::models::price::price_bar::PriceBar;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

use color_eyre::eyre::Result;

#[derive(Builder, Clone)]
#[builder(setter(prefix = "with"))]
pub struct ForwarderClient {
    client: reqwest::Client,
    end_point: String,
}

impl ForwarderClient {
    pub fn builder() -> ForwarderClientBuilder {
        ForwarderClientBuilder::default()
    }

    pub async fn get_stream(
        &self,
    ) -> eyre::Result<Pin<Box<dyn Stream<Item = eyre::Result<Option<DataEvent>>> + Send>>> {
        let response = self
            .client
            .get(self.end_point.clone())
            .send()
            .await?
            .bytes_stream()
            .map(|br| match br.map(parse_raw_data) {
                Ok(Ok(v)) => Ok(v),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(eyre::Report::new(e)),
            });

        let result = Box::pin(response);
        Ok(result)
    }
}

fn parse_raw_data(bytes: Bytes) -> Result<Option<DataEvent>> {
    let deserialized_data: PriceBar = serde_json::from_slice(&bytes)?;
    let data_event = DataEvent::PriceBar(deserialized_data);
    Ok(Some(data_event))
}
