
use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder};

use super::super::traits::Apply;
use crate::primatives::duration::NiceDuration;

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub struct Pool {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub idle_timeout: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_idle_per_host: Option<usize>,
}
impl Apply for Pool {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let mut b = b;
        b = match &self.idle_timeout {
            &Option::None => b,
            Option::Some(dur) => b.pool_idle_timeout(dur.get_duration()),
        };
        b = match &self.max_idle_per_host {
            &Option::None => b,
            Option::Some(max) => b.pool_max_idle_per_host(*max),
        };
        b
    }
}

