
use serde::{Serialize,Deserialize};
use hyper_util::server::conn::auto::{Http1Builder};

use config_crap::{NiceDuration};

#[derive(Clone,PartialEq,Eq,Serialize,Deserialize,Debug)]
pub struct Http1 {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub auto_date_header: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub half_close: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keep_alive: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub title_case_headers: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ignore_invalid_headers: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_headers: Option<usize>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub read_headers_timeout: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub writev: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_buf_size: Option<usize>,
}
impl Http1 {
    pub fn set_options<E>(this: &Option<Http1>, builder: &mut Http1Builder<'_,E>) {
        let this: &Self = match this {
            Option::None => return,
            Option::Some(this) => this,
        };
        if let Option::Some(x) = this.auto_date_header {
            builder.auto_date_header(x);
        }
        if let Option::Some(x) = this.half_close {
            builder.half_close(x);
        }
        if let Option::Some(x) = this.keep_alive {
            builder.keep_alive(x);
        }
        if let Option::Some(x) = this.title_case_headers {
            builder.title_case_headers(x);
        }
        if let Option::Some(x) = this.ignore_invalid_headers {
            builder.ignore_invalid_headers(x);
        }
        if let Option::Some(x) = this.max_headers {
            builder.max_headers(x);
        }
        if let Option::Some(x) = this.read_headers_timeout {
            builder.header_read_timeout(x.get_duration());
        }
        if let Option::Some(x) = this.writev {
            builder.writev(x);
        }
        if let Option::Some(x) = this.max_buf_size {
            builder.max_buf_size(x);
        }
    }
}
