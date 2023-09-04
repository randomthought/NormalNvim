use super::{models::QuoteResponse, utils};
use anyhow::{Context, Result};
use async_trait::async_trait;
use domain::{
    data::QouteProvider,
    models::{price::Quote, security::Security},
};
use std::io;

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
    async fn get_quote(&self, security: &Security) -> Result<Quote> {
        // let ticker = security.ticker;
        let url = format!(
            "https://api.polygon.io/v2/last/nbbo/{}?apiKey={}",
            security.ticker, self.api_key
        );
        let resp = self.client.get(url).send().await.unwrap();
        let qoute_response = resp.json::<QuoteResponse>().await.unwrap();
        let qoute = utils::to_quote(&qoute_response);
        Ok(qoute)
    }
}
