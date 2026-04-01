use std::{
    net::SocketAddr,
    collections::BTreeMap,
};

use anyhow::Context;
use serde::{Serialize,Deserialize};

use crate::{
    adapters::{
        path_helper::{GetTreePath,ServiceReqs,IntoServiceConfig,ServiceConfig},
        service_tree::{get_tree},
    },
    services::{
        openrouter::facades::settings::config::{OpenRouterSettingsConfig},
        openrouter::config::{OpenRouterConfiguration},
        web_request::config::{ClientLoader},
    },
};


#[derive(Deserialize,Clone,PartialEq,Debug)]
#[serde(tag = "type")]
pub enum ConfigMember {
    #[serde(alias="open-router", alias = "or")]
    OpenRouter(OpenRouterConfiguration),
    #[serde(alias="model-defaults")]
    ModelDefaults(OpenRouterSettingsConfig),
    #[serde(alias="http-client")]
    HttpClient(ClientLoader),
}

#[derive(Deserialize,Clone,PartialEq,Debug)]
pub struct Configuration {
    pub socket: SocketAddr,
    pub routes: BTreeMap<String,String>,
    pub services: Vec<ConfigMember>,
}
impl Configuration {

    /// Load the configuration from some path
    pub async fn load(path: String) -> anyhow::Result<Self> {
        let handle = tokio::task::spawn_blocking(move || -> anyhow::Result<Self> {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to open config file: '{}'", &path))?;
            Ok(serde_yaml::from_str(&text)?)
        });
        Ok(handle.await.unwrap()?)
    }
}
