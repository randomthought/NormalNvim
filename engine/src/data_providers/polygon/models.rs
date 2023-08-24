use serde::{Deserialize, Serialize};
// use serde_derive::Deserialize;
// use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[derive(Serialize, Deserialize, Debug)]
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
