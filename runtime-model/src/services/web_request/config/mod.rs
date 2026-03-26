use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder,Client,Error};

use crate::adapters::{
    path_helper::{GetTreePath,ServiceReqs,IntoServiceConfig,ServiceConfig},
    service_tree::{get_tree},
};

use super::service_impl::{load_client};


pub mod network;
use self::network::Networking;
pub mod http;
use self::http::{Http};
pub mod traits;
use self::traits::{Apply};

/// Actual config semantics for a `reqwest::Client`
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

/// Configuration that global machinery interacts with
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
impl IntoServiceConfig for ClientLoader {
    fn into_service_config(&self) -> ServiceConfig {
        ServiceConfig::new(self.clone())
    }
}
impl ServiceReqs for ClientLoader {

    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        self.path.get_tree_path()
    }

    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        Ok(Vec::new())
    }

    fn insert_to_tree(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=anyhow::Result<()>> + Send + 'static>> {
        let s = self.clone();
        Box::pin(async move {
            let path = s.path.clone();
            let tree = get_tree();
            if tree.contains_path(&path)? {
                tree.reload(&path, Box::new(s)).await?;
            } else {
                load_client(tree, s)?;
            }
            Ok(())
        })
    }
}
