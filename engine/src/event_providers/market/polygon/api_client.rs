use super::{models::QuoteResponse, utils};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use domain::{
    data::QouteProvider,
    models::{price::Quote, security::Security},
};
use futures_util::TryFutureExt;

pub struct ApiClient {
    api_key: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(api_key: String, client: reqwest::Client) -> Self {
        Self { api_key, client }
    }
}

#[async_trait]
impl QouteProvider for ApiClient {
    async fn get_quote(&self, security: &Security) -> Result<Quote, domain::error::Error> {
        let url = format!(
            "https://api.polygon.io/v2/last/nbbo/{}?apiKey={}",
            security.ticker, self.api_key
        );
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| domain::error::Error::Any(e.into()))?;

        let qoute_response = resp
            .json::<QuoteResponse>()
            .await
            .map_err(|e| domain::error::Error::Any(e.into()))?;

        let qoute =
            utils::to_quote(&qoute_response).map_err(|e| domain::error::Error::Message(e))?;

        Ok(qoute)
    }
}
