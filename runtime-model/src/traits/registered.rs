use std::{
    pin::Pin,
    future::Future,
    any::Any,
};

use tower_service::{Service};
use bytes::{
    Bytes,
};
use http::{
    request::{Request as HttpRequest},
    response::{Response as HttpResponse},
};
use hyper::body::Incoming;
use reqwest::{
    Request as HttpReqwestRequest,
    Response as HttpReqwestResponse,
};

use super::errors::{Err};

/// Generalized configuration
pub type BoxedConfig = Box<dyn Any +'static + Send + Send>;

/// Generalized future
pub type BoxedFuture<O> = Box<dyn Future<Output=O> + 'static + Send>;

/// A basic "this is a service" type of thing.
pub type ServiceObj<Req,Res,E> = Box<dyn Service<Req,Response=Res,Error=E,Future=Pin<Box<dyn Future<Output=Result<Res,E>> + 'static + Send>>>>;


#[derive(Clone,Copy,PartialEq,PartialOrd,Eq,Ord,Debug)]
#[repr(u8)]
pub enum ServiceKind {
    Meta = 0,
    HttpClient = 1,
    HttpServer = 2,
}


/// Represents an abstract service within the service mesh.
pub trait RegisteredService<E: Err>: 'static + Sync + Send {

    /// Priority is inverted highest values last
    fn get_priority(&self) -> usize { usize::MAX }

    /// Returns the roll this service futfills
    fn get_roles(&self) -> &'static [ServiceKind];

    /// Initialize a reload.
    fn reload<'a>(&'a mut self, config: BoxedConfig) -> Result<Pin<Box<dyn Future<Output=Result<(),E>> + 'a + Send>>,E>
    where
        E: Sized;

    /// Return a handle to an http client
    fn get_http_client(&self) -> Result<ServiceObj<HttpReqwestRequest,HttpReqwestResponse,E>,E>
    where
        E: Sized,
    {
        Err(<E as Err>::not_an_http_client::<Self>())
    }

    /// Return something cable to acting like an HTTP Server
    fn get_http_server(&self) -> Result<ServiceObj<HttpRequest<Incoming>,HttpResponse<Bytes>,E>,E>
    where
        E: Sized,
    {
        Err(<E as Err>::not_an_http_server::<Self>())
    }
}
