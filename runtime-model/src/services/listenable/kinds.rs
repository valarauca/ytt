use anyhow::Error;
use http_body::{Body};
use hyper::body::Incoming;
use bytes::{Bytes};
use http::{Request,Response};

pub type ExtHttpRequest = Request<Incoming>;
pub type ExtHttpResponse = Response<Box<dyn Body<Data=Bytes,Error=Error> + 'static + Send>>;
