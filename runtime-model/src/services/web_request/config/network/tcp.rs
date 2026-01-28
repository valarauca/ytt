use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder};

use crate::primatives::duration::NiceDuration;
use super::super::traits::Apply;

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub struct Tcp {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keepalive_duration: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keepalive_interval: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keepalive_retries: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub no_delay: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub user_timeout: Option<NiceDuration>,
}
impl Apply for Tcp {

    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let mut b = b;
        b = match &self.keepalive_duration {
            &Option::None => b,
            Option::Some(dur) => b.tcp_keepalive(dur.get_duration()),
        };
        b = match &self.keepalive_interval {
            &Option::None => b,
            Option::Some(dur) => b.tcp_keepalive_interval(dur.get_duration()),
        };
        b = match &self.keepalive_retries {
            &Option::None => b,
            Option::Some(count) => b.tcp_keepalive_retries(*count),
        };
        b = match &self.no_delay {
            &Option::None => b,
            Option::Some(x) => b.tcp_nodelay(*x),
        };

        #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
        fn set_tcp_user_timeout(b: ClientBuilder, arg: &NiceDuration) -> ClientBuilder {
            b.tcp_user_timeout(arg.get_duration())
        }
        #[cfg(not(any(target_os = "android", target_os = "fuchsia", target_os = "linux")))]
        fn set_tcp_user_timeout(b: ClientBuilder, _arg: &NiceDuration) -> ClientBuilder {
            b
        }

        b = match &self.user_timeout {
            &Option::None => b,
            Option::Some(dur) => set_tcp_user_timeout(b, dur),
        };
        b
    }
}

