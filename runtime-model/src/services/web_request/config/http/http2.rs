use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder};

use super::super::traits::Apply;
use crate::primatives::duration::NiceDuration;

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub struct Http2 {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub initial_stream_window_size: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub initial_connection_window_size: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub adaptive_window: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_frame_size: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_header_list_size: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keep_alive_interval: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keep_alive_timeout: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keep_alive_while_idle: Option<bool>,
}
impl Apply for Http2 {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let mut b = b;

        if let Option::Some(x) = &self.initial_stream_window_size {
            b = b.http2_initial_stream_window_size(*x);
        }
        if let Option::Some(x) = &self.initial_connection_window_size {
            b = b.http2_initial_connection_window_size(*x);
        }
        if let Option::Some(x) = &self.adaptive_window {
            b = b.http2_adaptive_window(*x);
        }
        if let Option::Some(x) = &self.max_frame_size {
            b = b.http2_max_frame_size(*x);
        }
        if let Option::Some(x) = &self.max_header_list_size {
            b = b.http2_max_header_list_size(*x);
        }
        if let Option::Some(x) = &self.keep_alive_interval {
            b = b.http2_keep_alive_interval(x.get_duration());
        }
        if let Option::Some(x) = &self.keep_alive_timeout {
            b = b.http2_keep_alive_timeout(x.get_duration());
        }
        if let Option::Some(x) = &self.keep_alive_while_idle {
            b = b.http2_keep_alive_while_idle(*x);
        }

        b
    }
}
