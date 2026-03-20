
use serde::{Serialize,Deserialize};

use runtime_model::services::web_request::config::{ClientConfig};

use crate::endpoints::config::RouterConfig;

#[derive(Serialize,Deserialize,Clone,Debug,PartialEq)]
#[serde(transparent)]
pub struct Config {
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


impl Config {

    fn get_clients(&self) -> impl Iterator<Item=&ClientConfig> {
        self.list.iter().filter_map(|x| match x {
            ConfigEntry::Client(c) => Some(c),
            _ => None,
        })
    }

    fn get_listener(&self) -> impl Iterator<Item=&RouterConfig> {
        self.list.iter().filter_map(|x| match x {
            ConfigEntry::Listener(r) => Some(r),
            _ => None,
        })
    }
}


