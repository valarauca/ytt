
use serde::{Deserialize,Serialize};

use crate::services::openrouter::facades::settings::config::{ModelDefaults};
use openrouter::completions::request::FunctionDescription;

#[derive(Clone,Serialize,Deserialize,PartialEq,Debug)]
pub struct Tool {
    pub client: String,
    pub path: String,
    pub desc: FunctionDescription,
    pub info: Schemantics,
}

#[derive(Clone,Serialize,Deserialize,PartialEq,Debug)]
#[serde(tag = "type")]
pub enum Schemantics {
    #[serde(rename = "open-router", alias = "or")]
    OpenRouter(ModelDefaults),
}

