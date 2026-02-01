use axum::body::Body;
use http::{Request,Response};

pub type ExtHttpRequest = Request<Body>;
pub type ExtHttpResponse = Response<Body>;
