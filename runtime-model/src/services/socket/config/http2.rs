
use serde::{Serialize,Deserialize};
use hyper_util::server::conn::auto::{Http2Builder};
use either::Either;

use config_crap::{NiceDuration};



#[derive(Clone,PartialEq,Eq,Serialize,Deserialize,Debug)]
pub struct Http2Keepalive {
    pub interval: NiceDuration,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub timeout: Option<NiceDuration>,
}

#[derive(Clone,PartialEq,Eq,Serialize,Deserialize,Debug)]
pub struct DefaultFlowControl {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub initial_connection_window_size: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub initial_stream_window_size: Option<u32>,
}

#[derive(Clone,PartialEq,Eq,Serialize,Deserialize,Debug)]
pub struct Http2 {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub auto_date_header: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_pending_accept_reset_streams: Option<usize>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_local_error_reset_streams: Option<usize>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_frame_size: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_concurrent_streams: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_header_list_size: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_send_buffer_size: Option<usize>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub adaptive_flow_control: Option<Either<bool,DefaultFlowControl>>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keep_alive: Option<Http2Keepalive>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub enable_connect_protocol: Option<bool>,
}
impl Http2 {
    pub fn set_options<E>(this: &Option<Http2>, builder: &mut Http2Builder<'_,E>) {
        let this: &Self = match this {
            Option::None => return,
            Option::Some(this) => this,
        };
        if let Option::Some(x) = this.auto_date_header {
            builder.auto_date_header(x);
        }
        if let Option::Some(x) = this.max_send_buffer_size {
            builder.max_send_buf_size(x);
        }
        if let Option::Some(x) = this.max_pending_accept_reset_streams {
            builder.max_pending_accept_reset_streams(x);
        }
        if let Option::Some(x) = this.max_local_error_reset_streams {
            builder.max_local_error_reset_streams(x);
        }
        if let Option::Some(x) = this.max_frame_size {
            builder.max_frame_size(x);
        }
        if let Option::Some(x) = this.max_concurrent_streams {
            builder.max_concurrent_streams(x);
        }
        if let Option::Some(x) = this.max_header_list_size {
            builder.max_header_list_size(x);
        }
        if let Option::Some(flow_control) = &this.adaptive_flow_control {
            match flow_control {
                Either::Left(b) => {
                    builder.adaptive_window(*b);
                },
                Either::Right(inner) => {
                    builder.adaptive_window(false);
                    if let Option::Some(x) = inner.initial_connection_window_size {
                        builder.initial_connection_window_size(x);
                    }
                    if let Option::Some(x) = inner.initial_stream_window_size {
                        builder.initial_stream_window_size(x);
                    }
                }
            };
        }
        if let Option::Some(keepalive) = &this.keep_alive {
            builder.keep_alive_interval(keepalive.interval.get_duration());
            if let Option::Some(timeout) = keepalive.timeout {
                builder.keep_alive_timeout(timeout.get_duration());
            }
        }
        if let Option::Some(true) = this.enable_connect_protocol {
            builder.enable_connect_protocol();
        }
    }
}
