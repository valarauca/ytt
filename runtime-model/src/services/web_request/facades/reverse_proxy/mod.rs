use serde::{Deserialize,Serialize};

use super::super::config::{ClientConfig};


#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Default)]
pub struct ReverseProxy {
    pub client: DefaultClient,
}

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Default)]
#[serde(untagged)]
pub enum DefaultClient {
    Path(String),
    Config(ClientConfig),
}
impl Default for DefaultClient {
    fn default() -> Self {
        Self::Path("default/client".to_string()),
    }
}

/*
pub enum ReverseProxyRoute {
    Fixed {

    }
}
*/
