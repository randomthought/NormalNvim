use std::{collections::HashMap, u64};

use anyhow::Result;
use async_trait::async_trait;
use domain::{
    data::QouteProvider,
    models::{
        price::{PriceHistory, Quote},
        security::Security,
    },
};
use futures_util::Stream;

pub struct BackTestingConfig {
    pub slipage: u64,
    pub spread: u64,
}

pub struct FileProvider<'a> {
    pub files: Vec<Box<str>>,
    symbol_quotes: HashMap<&'a Security, Quote>,
}

impl<'a> FileProvider<'a> {
    fn new(files: Vec<Box<str>>) -> Self {
        Self {
            files,
            symbol_quotes: HashMap::new(),
        }
    }
}

#[async_trait]
impl<'a> QouteProvider for FileProvider<'a> {
    async fn get_quote(&self, security: &Security) -> Result<Quote> {
        // let s = self.symbol_quotes.get(security);
        todo!()
    }
}
