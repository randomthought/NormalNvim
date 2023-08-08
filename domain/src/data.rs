use crate::models::{price::Quote, security::Security};
use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait DataProvider {
    async fn get_quote(&self, security: &Security) -> Result<Quote, io::Error>;
}
