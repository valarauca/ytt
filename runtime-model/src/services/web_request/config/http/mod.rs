use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder};

use super::traits::Apply;

pub mod misc;
use self::misc::MiscPolicy;
pub mod http1;
use self::http1::Http1;
pub mod http2;
use self::http2::Http2;

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub enum OnlyVersion {
    #[serde(rename = "http_v1")]
    V1,
    #[serde(rename = "http_v2")]
    V2
}

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Default)]
pub struct Http {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub only: Option<OnlyVersion>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub v1: Option<Http1>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub v2: Option<Http2>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub misc: Option<MiscPolicy>,
}
impl Apply for Http {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let b = match &self.only {
            &Option::Some(OnlyVersion::V1) => {
                Http1::apply(&self.v1, b.http1_only())
            }
            _ => {
                Http1::apply(&self.v1, Http2::apply(&self.v2, b))
            }
        };
        let b = MiscPolicy::apply(&self.misc, b);

        b
    }
}
