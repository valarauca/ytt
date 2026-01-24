
use std::net::IpAddr;

use mirror_mirror::{Reflect};
use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder};
use crate::generic_config::client::traits::{Apply};

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Reflect)]
pub struct Bind {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub address: Option<IpAddr>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub interface: Option<String>,
}
impl Apply for Bind {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {

        /*
         * Fun Fact:
         *    `if cfg!(` will still do type checking on data
         *    that is cfg'd out. 
         *
         *    Meaning if a function isn't defined on your target
         *    OS and it appears under one of those cfg arms.
         *    Compilation fails.
         *
         *    So instead we have to do this ugly crap
         */

        #[cfg(target_family = "unix")]
        fn set_interface(cb: ClientBuilder, arg: &str) -> ClientBuilder {
            cb.interface(arg)
        }

        #[cfg(not(target_family = "unix"))]
        fn set_interface(cb: ClientBuilder, _arg: &str) -> ClientBuilder {
            cb
        }

        let b = match &self.interface {
            Option::None => b,
            Option::Some(interface) => {
                set_interface(b, interface)
            }
        };
        match &self.address {
            Option::None => b,
            Option::Some(local) => b.local_address(*local),
        }
    }
}
