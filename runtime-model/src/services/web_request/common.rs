use std::{
    pin::Pin,
    future::Future,
};
use tower::{Service};
use reqwest::{Request,Response};

pub type BoxedHttpResponse<E> = Pin<Box<dyn Future<Output=Result<Response,E>> + Send + 'static>>;
pub type HttpClientObj<E> = Box<dyn Service<Request,Response=Response,Error=E,Future=Pin<Box<dyn Future<Output=Result<Response,E>> + Send + 'static>>> + Send + 'static>;
