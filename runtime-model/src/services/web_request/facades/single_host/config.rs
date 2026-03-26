use serde::{Deserialize,Serialize};


#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Default)]
pub struct SingleHostReverseProxy {
    pub client: String,
    pub path: String,
    pub name: String,
}

