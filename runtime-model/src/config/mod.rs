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
