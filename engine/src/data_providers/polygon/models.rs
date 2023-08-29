use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aggregates {
    pub ev: String,
    pub sym: String,
    pub v: u64,
    pub av: i64,
    pub op: f64,
    pub vw: f64,
    pub o: f64,
    pub c: f64,
    pub h: f64,
    pub l: f64,
    pub a: f64,
    pub z: f64,
    pub s: u64,
    pub e: u64,
    #[serde(default)]
    pub otc: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMessage {
    pub ev: String,
    pub status: String,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(rename = "request_id")]
    pub request_id: String,
    pub results: Quote,
    pub status: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    #[serde(rename = "P")]
    pub p: f64,
    #[serde(rename = "S")]
    pub s: u64,
    #[serde(rename = "T")]
    pub t: String,
    #[serde(rename = "X")]
    pub x: i64,
    #[serde(rename = "p")]
    pub p2: f64,
    pub q: i64,
    #[serde(rename = "s")]
    pub s2: u64,
    #[serde(rename = "t")]
    pub t2: u64,
    #[serde(rename = "x")]
    pub x2: i64,
    pub y: i64,
    pub z: i64,
}
