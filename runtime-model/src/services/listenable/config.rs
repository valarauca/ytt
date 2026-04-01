use std::{
    net::SocketAddr,
    collections::BTreeMap,
};


#[derive(Deserialize,Clone,PartialEq,Debug)]
pub struct ListenerConfig {
    pub path: String,
    pub socket: SocketAddr,
    pub routes: BTreeMap<String,String>,
}
