
use serde::{Serialize,Deserialize};

use runtime_model::services::web_request::config::{ClientConfig};
use crate::endpoints::config::RouterConfig;

#[derive(Serialize,Deserialize,Clone,Debug,PartialEq)]
pub struct Config {
    // ensures 'some client' configuration always exists
    #[serde(default)]
    default_client: ClientConfig,

    list: Vec<ConfigEntry>,
}


#[derive(Serialize,Deserialize,Clone,Debug,PartialEq)]
#[serde(tag = "type")]
pub enum ConfigEntry {
    #[serde(rename = "client")]
    Client(ClientConfig),
    #[serde(rename = "listener")]
    Listener(RouterConfig),
}
