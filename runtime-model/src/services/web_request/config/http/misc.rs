use serde::{Deserialize,Serialize};
use mirror_mirror::{Reflect};
use reqwest::{ClientBuilder};

use crate::generic_config::client::traits::{Apply};
use crate::primatives::headers::HHeaderValue;


#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Reflect)]
pub struct MiscPolicy {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub send_referer: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub allow_http09_response: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub follow_redirects: Option<RedirectPolicy>,
    #[serde(default,skip_serializing_if="bool_is_false")]
    pub no_gzip: bool,
    #[serde(default,skip_serializing_if="bool_is_false")]
    pub no_brotli: bool,
    #[serde(default,skip_serializing_if="bool_is_false")]
    pub no_zstd: bool,
    #[serde(default,skip_serializing_if="bool_is_false")]
    pub no_deflate: bool,
    #[serde(
        default,
        skip_serializing_if="Option::is_none",
    )]
    pub default_user_agent: Option<HHeaderValue>,
}

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub enum RedirectPolicy {
    Limit(usize),
    #[serde(alias = "never", alias="disabled")]
    Never,
}
impl Apply for MiscPolicy {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let mut b = b;
        
        if let Option::Some(val) = &self.default_user_agent {
            b = b.user_agent(val.0.clone());
        }

        b = match &self.follow_redirects {
            &Option::Some(RedirectPolicy::Limit(limit)) => {
                b.redirect(reqwest::redirect::Policy::limited(limit))
            }
            &Option::Some(RedirectPolicy::Never) => {
                b.redirect(reqwest::redirect::Policy::none())
            },
            _ => b,
        };
        if let Option::Some(false) = &self.send_referer {
            b = b.referer(false);
        }
        if let Option::Some(true) = &self.allow_http09_response {
            b = b.http09_responses();
        }

        if self.no_gzip {
            b = b.no_gzip();
        }
        if self.no_brotli {
            b = b.no_brotli();
        }
        if self.no_zstd {
            b = b.no_zstd();
        }
        if self.no_deflate {
            b = b.no_deflate();
        }

        b
    }
}

// serde needs a function to call
fn bool_is_false(x: &bool) -> bool {
    !*x
}
