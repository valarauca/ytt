use serde::{Deserialize,Serialize};
use mirror_mirror::{Reflect};
use reqwest::{ClientBuilder};

use crate::generic_config::client::traits::{Apply};

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Reflect)]
pub struct Http1 {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub allow_obsolete_multiline_headers_in_response: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ignore_invalid_headers_in_response: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub allow_spaces_after_header_name_in_response: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub title_case_headers: Option<bool>,
}

impl Apply for Http1 {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let mut b = b;

        if let Option::Some(flag) = &self.allow_obsolete_multiline_headers_in_response {
            b = b.http1_allow_obsolete_multiline_headers_in_responses(*flag);
        }
        if let Option::Some(flag) = &self.ignore_invalid_headers_in_response {
            b = b.http1_ignore_invalid_headers_in_responses(*flag);
        }
        if let Option::Some(flag) = &self.allow_spaces_after_header_name_in_response {
            b = b.http1_allow_spaces_after_header_name_in_responses(*flag);
        }
        if let Option::Some(true) = &self.title_case_headers {
            b = b.http1_title_case_headers();
        }
        b
    }
}
