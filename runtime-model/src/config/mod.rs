use std::{
    net::SocketAddr,
    collections::BTreeMap,
};

use anyhow::Context;
use serde::{Deserialize};

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

pub mod runtime;


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
    #[serde(default)]
    pub rt: self::runtime::TokioRuntime,
    pub services: Vec<ConfigMember>,
}
impl Configuration {

    /*
     * Initially the configuration is loaded when the program is running synchronously
     * as the configuration specifies how to setup the runtime
     *
     * It therefore has some sync and async interfaces.
     *
     */

    /// Perform a synchronous load
    pub fn sync_load(path: String) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to open config file: '{}'", &path))?;
        Ok(serde_yaml::from_str(&text)?)
    }

    /// Create a runtime from the defined configuration
    pub fn make_rt(&self) -> Result<tokio::runtime::Runtime,std::io::Error> {
        self.rt.build()
    }

    /// Load the configuration from some path
    pub async fn load(path: String) -> anyhow::Result<Self> {
        let handle = tokio::task::spawn_blocking(move || -> anyhow::Result<Self> {
            Self::sync_load(path)
        });
        Ok(handle.await.unwrap()?)
    }
}
