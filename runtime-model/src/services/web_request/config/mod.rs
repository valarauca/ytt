use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder,Client,Error};

pub mod network;
use self::network::Networking;
pub mod http;
use self::http::{Http};
pub mod traits;
use self::traits::{Apply};

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Default)]
pub struct ClientConfig {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub network: Option<Networking>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub protocol: Option<Http>,
}
impl ClientConfig {
    pub fn build(&self) -> Result<Client,Error> {
        let b = ClientBuilder::new();
        let b = b.user_agent(concat!("yttrium", "/", "0.1"));
        self.apply_opts(b).build()
    }
}
impl Apply for ClientConfig {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let b = Networking::apply(&self.network, b);
        Http::apply(&self.protocol, b)
    }
}

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub struct ClientLoader {
    pub(crate) path: String,
    pub(crate) buffer: usize,
    pub(crate) config: ClientConfig,
}
impl Default for ClientLoader {
    fn default() -> ClientLoader {
        Self {
            path: "/default/client".to_string(),
            buffer: 1,
            config: ClientConfig::default(),
        }
    }
}
