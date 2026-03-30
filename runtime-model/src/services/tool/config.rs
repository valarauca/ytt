use std::collections::BTreeMap;

use serde::{Deserialize,Serialize};
use url::Url;

use crate::services::openrouter::facades::settings::config::{ModelDefaults};
use crate::primatives::headers::{HHeaderName};

use openrouter::completions::request::FunctionDescription;
use config_crap::template::string_or_template::{StringOrTemplate};
use config_crap::boolean::{Boolean};

#[derive(Clone,Deserialize,PartialEq,Debug)]
pub struct ToolConfig {
    pub client: String,
    pub path: String,
    pub desc: FunctionDescription,
    pub info: Schemantics,
}

/// Where the calls are made too
#[derive(Clone,Deserialize,PartialEq,Debug)]
#[serde(tag = "type")]
pub enum Schemantics {
    #[serde(rename = "open-router", alias = "or")]
    OpenRouter {
        model_defaults: ModelDefaults,
    },
    HttpGet {
        url: StringOrTemplate,
        //headers: BTreeMap<
    },
}

pub struct ToolMiddlwareConfig {
    /// Where this middleware will be inserted into the tree
    pub path: String,
    /// Path to internal LLM API
    pub llm_api_path: String,
    /// Mapping of `tool_name` -> `/internal/too/path`
    pub tool_map: BTreeMap<String,String>,
    pub serialize: Option<Boolean>,
}


