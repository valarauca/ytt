use serde::{Deserialize,Serialize};
use serde::de::{self};
use reqwest::{ClientBuilder};
use reqwest::tls::Version;

use super::super::traits::Apply;

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub struct Tls {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub https_only: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub sni: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub send_tls_info: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub accept_dangerious_invalid_hostnames: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub accept_dangerious_invalid_certs: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub min_tls: Option<VersionWrapper>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_tls: Option<VersionWrapper>,
}
impl Apply for Tls {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let mut b = b;
        b = match &self.https_only {
            &Option::None => b,
            &Option::Some(flag) => b.https_only(flag),
        };
        b = match &self.sni {
            &Option::None => b,
            &Option::Some(flag) => b.tls_sni(flag),
        };
        b = match &self.send_tls_info {
            &Option::None => b,
            &Option::Some(flag) => b.tls_info(flag),
        };
        b = match &self.accept_dangerious_invalid_hostnames {
            &Option::None => b,
            &Option::Some(flag) => b.danger_accept_invalid_hostnames(flag),
        };
        b = match &self.accept_dangerious_invalid_certs {
            &Option::None => b,
            &Option::Some(flag) => b.danger_accept_invalid_certs(flag),
        };
        b = match &self.min_tls {
            &Option::None => b,
            &Option::Some(ref min) => b.min_tls_version(min.data.clone()),
        };
        b = match &self.max_tls {
            &Option::None => b,
            &Option::Some(ref max) => b.max_tls_version(max.data.clone()),
        };
        b
    }
}


#[derive(Clone,PartialEq,Eq,Debug)]
pub struct VersionWrapper {
    pub data: Version,
}
impl Default for VersionWrapper {
    fn default() -> Self {
        Self { data: Version::TLS_1_3 }
    }
}
impl Serialize for VersionWrapper {
    fn serialize<S>(&self, s: S) -> Result<S::Ok,S::Error>
    where
        S: serde::ser::Serializer,
    {
        if self.data == Version::TLS_1_0 {
            s.serialize_str("tlsv1.0")
        } else if self.data == Version::TLS_1_1 {
            s.serialize_str("tlsv1.1")
        } else if self.data == Version::TLS_1_2 {
            s.serialize_str("tlsv1.2")
        } else if self.data == Version::TLS_1_3 {
            s.serialize_str("tlsv1.3")
        } else {
            panic!("valid TLS version")
        }
    }
}

struct VersionVisitor;
impl<'de> de::Visitor<'de> for VersionVisitor {
    type Value = VersionWrapper;
    fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(fmt, "a string containing: [ 'tlsv1.0', 'tlsv1.1', 'tlsv1.2', 'tlsv1.3' ]")
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value,E> {
        let data = match v.trim() {
            "tlsv1.0" => Version::TLS_1_0,
            "tlsv1.1" => Version::TLS_1_1,
            "tlsv1.2" => Version::TLS_1_2,
            "tlsv1.3" => Version::TLS_1_3,
            x => return Err(E::invalid_value(de::Unexpected::Str(x), &self)),
        };
        Ok(VersionWrapper { data })
    }
}
impl<'de> de::Deserialize<'de> for VersionWrapper {
    fn deserialize<D>(d: D) -> Result<Self,D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(VersionVisitor)
    }
}
