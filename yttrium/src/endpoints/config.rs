use std::{
    collections::{BTreeMap,BTreeSet},
    net::SocketAddr,
};
use serde::{Serialize,Deserialize};


#[derive(Serialize,Deserialize,PartialEq,Clone,Debug)]
pub struct RouterConfig {
    pub routes: BTreeMap<String, EndPoint>,
    pub socket: SocketAddr,
}

#[derive(Serialize,Deserialize,PartialEq,Clone,Debug)]
pub struct EndPoint {
    pub supports: BTreeSet<Method>,
    pub tree_location: Vec<String>,
}
impl EndPoint {
    pub fn build_filter(&self) -> axum::routing::MethodFilter {
        let mut supports = self.supports.iter().map(|x| axum::routing::MethodFilter::from(*x));
        let init = supports.next().unwrap_or(axum::routing::MethodFilter::GET);
        supports.fold(init, |a,b| a.or(b))
    }
    pub fn tree_path<'a>(&'a self) -> Vec<&'a str> {
        self.tree_location.iter().map(|x| x.as_str()).collect()
    }
}

#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug,Serialize,Deserialize)]
#[repr(u8)]
pub enum Method {
    Get = 1,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}
impl From<Method> for http::method::Method {
    fn from(m: Method) -> Self {
        match m {
            Method::Get => http::method::Method::GET,
            Method::Post => http::method::Method::POST,
            Method::Put => http::method::Method::PUT,
            Method::Delete => http::method::Method::DELETE,
            Method::Head => http::method::Method::HEAD,
            Method::Options => http::method::Method::OPTIONS,
            Method::Connect => http::method::Method::CONNECT,
            Method::Patch => http::method::Method::PATCH,
            Method::Trace => http::method::Method::TRACE,
        }
    }
}
impl From<Method> for axum::routing::MethodFilter {
    fn from(m: Method) -> Self {
        match m {
            Method::Get => axum::routing::MethodFilter::GET,
            Method::Post => axum::routing::MethodFilter::POST,
            Method::Put => axum::routing::MethodFilter::PUT,
            Method::Delete => axum::routing::MethodFilter::DELETE,
            Method::Head => axum::routing::MethodFilter::HEAD,
            Method::Options => axum::routing::MethodFilter::OPTIONS,
            Method::Connect => axum::routing::MethodFilter::CONNECT,
            Method::Patch => axum::routing::MethodFilter::PATCH,
            Method::Trace => axum::routing::MethodFilter::TRACE,
        }
    }
}
