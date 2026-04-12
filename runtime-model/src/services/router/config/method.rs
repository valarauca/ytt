use std::{
    collections::{BTreeSet},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum Method {
    Connect = 1,
    Delete = 2,
    Get = 3,
    Head = 4,
    Options = 5,
    Patch = 6,
    Post = 7,
    Put = 8,
    Trace = 9,
}
impl Method {


	pub fn from_http(m: &http::Method) -> Option<Self> {
	    match m {
	        &http::Method::CONNECT => Some(Self::Connect),
	        &http::Method::DELETE => Some(Self::Delete),
	        &http::Method::GET => Some(Self::Get),
	        &http::Method::HEAD => Some(Self::Head),
	        &http::Method::OPTIONS => Some(Self::Options),
	        &http::Method::PATCH => Some(Self::Patch),
	        &http::Method::POST => Some(Self::Post),
	        &http::Method::PUT => Some(Self::Put),
	        &http::Method::TRACE => Some(Self::Trace),
	        _ => None,
	    }
	}

    pub const ALL: &[Method] = &[
        Method::Connect,
        Method::Delete,
        Method::Get,
        Method::Head,
        Method::Options,
        Method::Patch,
        Method::Post,
        Method::Put,
        Method::Trace,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Method::Connect => "CONNECT",
            Method::Delete => "DELETE",
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Patch => "PATCH",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Trace => "TRACE",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "CONNECT" => Some(Method::Connect),
            "DELETE" => Some(Method::Delete),
            "GET" => Some(Method::Get),
            "HEAD" => Some(Method::Head),
            "OPTIONS" => Some(Method::Options),
            "PATCH" => Some(Method::Patch),
            "POST" => Some(Method::Post),
            "PUT" => Some(Method::Put),
            "TRACE" => Some(Method::Trace),
			_ => None,
        }
    }

    fn from_str_serde<'de,E>(s: &str) -> Result<Self,E>
    where
        E: serde::de::Error,
    {

        Self::from_str(s)
            .ok_or_else(|| E::unknown_variant(s, &["CONNECT","DELETE","GET","HEAD","OPTIONS","PATCH","POST","PUT","TRACE"]))
    }

    pub fn parse_method_key<'de,E>(key: &str) -> Result<BTreeSet<Self>,E>
    where
        E: serde::de::Error,
    {
        use serde::de::{Unexpected};

        let key = key.trim();
        if key == "*" {
            return Ok(Method::ALL.iter().copied().collect());
        }
        let mut set = BTreeSet::new();
        for part in key.split('|') {
            let method = Self::from_str_serde::<E>(part)?;
            set.insert(method);
        }
        if set.is_empty() {
            return Err(E::invalid_value(Unexpected::Str(key),&"key must contain 1 or more valid HTTP verbs"));
        }
        Ok(set) 
    }
}
impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

