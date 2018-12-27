use super::*;
use super::super::utils::hex_to;
use serde::{Deserialize, Serialize};
use bytes::Bytes;

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    pub id: Option<u32>,
    pub method: String,
    pub params: Params,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Respond {
    pub id: Option<u32>,
    pub result: ResultOf,
    pub error: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Params {
    Work(Work),
    Bool(bool),
    Difficulty([u32; 1]),
    User([String; 2]),
    None(Vec<()>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ResultOf {
    Authorize(bool),
    Subscribe(ResultOfSubscribe),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultOfSubscribe(
    pub [StringWithBytes; 2],   // set_difficulty & notify
    #[serde(deserialize_with = "hex_to::bytes")]
    pub Bytes,                  // xnonce1
    pub u32,                    // xnonce2_size
);

#[derive(Serialize, Deserialize, Debug)]
pub struct StringWithBytes(
    String,
    #[serde(deserialize_with = "hex_to::bytes")]
    Bytes,
);

pub trait ToString: serde::Serialize {
    fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}

impl<T: serde::Serialize> ToString for T {}