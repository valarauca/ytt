use std::{
    net::SocketAddr,
    collections::BTreeMap,
};


#[derive(Deserialize,Clone,PartialEq,Debug)]
pub struct ListenerConfig {
    pub socket: SocketAddr,
}
