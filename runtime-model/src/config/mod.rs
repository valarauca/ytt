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
        web_request::facades::single_host::config::{SingleHostReverseProxyConfig},
        router::config::{RouterConfig},
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
    #[serde(alias="single-host-reverse-proxy")]
    SingleHostReverseProxy(SingleHostReverseProxyConfig),
    #[serde(alias="router")]
    Router(RouterConfig),
}
impl ServiceReqs for ConfigMember {
    /// Where this service will be inserted into the tree
    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        match self {
            Self::OpenRouter(a) => a.creates(),
            Self::ModelDefaults(a) => a.creates(),
            Self::HttpClient(a) => a.creates(),
            Self::SingleHostReverseProxy(a) => a.creates(),
            Self::Router(a) => a.creates(),
        }
    }

    /// What this service needs to operate
    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        match self {
            Self::OpenRouter(a) => a.requires(),
            Self::ModelDefaults(a) => a.requires(),
            Self::HttpClient(a) => a.requires(),
            Self::SingleHostReverseProxy(a) => a.requires(),
            Self::Router(a) => a.requires(),
        }
    }

    /// Inser this type into the tree
    fn insert_to_tree(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=anyhow::Result<()>> + Send + 'static>> {
        match self {
            Self::OpenRouter(a) => a.insert_to_tree(),
            Self::ModelDefaults(a) => a.insert_to_tree(),
            Self::HttpClient(a) => a.insert_to_tree(),
            Self::SingleHostReverseProxy(a) => a.insert_to_tree(),
            Self::Router(a) => a.insert_to_tree(),
        }
    }
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
