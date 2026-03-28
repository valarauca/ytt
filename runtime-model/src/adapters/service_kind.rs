use std::any::Any;

use crate::{
    traits::{BoxedConfig, not_an_http_client, not_an_http_server, not_a_lua_runtime},
    adapters::reconfigurable::{Reconfig,ReconfigurableService},
    adapters::s3service::{BoxCloneSyncService},
    services::listenable::{ kinds::{ExtHttpRequest,ExtHttpResponse}, webshit::{Webshit}},
    services::lua::repr::{LuaCall},
};
use serde_json::{Value as JSValue};
use http::{Request as HttpRequest, Response as HttpResponse};
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};
use axum::body::{Body as AxumBody};
use hyper::body::{Incoming as HyperBody};
use http_body_util::{BodyExt, combinators::BoxBody};
use openrouter::completions::{Request as ORRequest,Response as ORResponse};


/// Interior service definations
///
/// Handles deligating way the correct request/response
/// type is for an undelrying service
#[non_exhaustive]
pub enum ServiceManagement {
    OpenRouter(Box<dyn Reconfig<ORRequest,ORResponse> + 'static>),
    EndPoint(Box<dyn Reconfig<ExtHttpRequest,ExtHttpResponse> + 'static>),
    WebClient(Box<dyn Reconfig<ReqwestRequest,ReqwestResponse> + 'static>),
    LuaRuntime(Box<dyn Reconfig<LuaCall,JSValue> + 'static>),
}
impl<C> From<ReconfigurableService<C,LuaCall,JSValue>> for ServiceManagement
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,LuaCall,JSValue>) -> Self {
        Self::LuaRuntime(Box::new(item))
    }
}
impl<C> From<ReconfigurableService<C,ReqwestRequest,ReqwestResponse>> for ServiceManagement
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,ReqwestRequest,ReqwestResponse>) -> Self {
        Self::WebClient(Box::new(item))
    }
}
impl<C> From<ReconfigurableService<C,ExtHttpRequest,ExtHttpResponse>> for ServiceManagement
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,ExtHttpRequest,ExtHttpResponse>) -> Self {
        Self::EndPoint(Box::new(item))
    }
}
impl<C> From<ReconfigurableService<C,ORRequest,ORResponse>> for ServiceManagement
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,ORRequest,ORResponse>) -> Self {
        Self::OpenRouter(Box::new(item))
    }
}

/// Concurrent handle to the internal & reconfigurable `reqwest::Client` object.
///
/// This type is very cheap to clone. Behind the scenes it is effectively an
///
/// `mpsc::Sender<(oneshot::Sender<Request>,oneshot::Receiver<Result<Response,Error>>)>`
///
/// The goal is roughly to ensure the underlying `reqwest::Client` can be dynamically
/// re-configured if need be, thusly client requests are totally decoupled from a handle
/// to the client. 
pub type ReqwestClientHandle = BoxCloneSyncService<ReqwestRequest,ReqwestResponse,anyhow::Error>;
/*
/// See note on [`ReqwestClientHandle`].
///
/// The only difference is this abstracts away the conversion into `Axum`'s request type.
pub type AxumClientHandle = BoxCloneSyncService<HttpRequest<AxumBody>,HttpResponse<AxumBody>,anyhow::Error>;

/// See note on [`ReqwestClientHandle`].
///
/// The only difference is this abstracts away the conversion into hyper's core type(s).
pub type HyperClientHandle = BoxCloneSyncService<HttpRequest<HyperBody>,HttpResponse<StreamingBody>,anyhow::Error>;
*/

impl ServiceManagement {

    /// Central reloading
    pub async fn reload(&self, config: BoxedConfig) -> Result<(),anyhow::Error> {
        match self {
            Self::OpenRouter(client) => client.reconfig(config).await,
            Self::EndPoint(client) => client.reconfig(config).await,
            Self::WebClient(client) => client.reconfig(config).await,
            Self::LuaRuntime(client) => client.reconfig(config).await,
        }
    }

    /*
     * Web Client
     * Used internally to make external requests.
     * 
     * Internally everything is routed to `reqwest`. The individual per-type methods
     * exist so the external API we present is type safe and the type conversions
     * can happen concurrently within middleware.
     *
     */

    /// Return the queried path location as a handle to the reqwest client
    pub fn get_reqwest_web_client(&self) -> anyhow::Result<ReqwestClientHandle> {
        match self {
            Self::WebClient(client) => Ok(client.get_service()),
            _ => Err(not_an_http_client::<Self>()),
        }
    }

    /*
     * API for specifically interacting with an OpenRouter client.
     *
     *
     */

    pub fn get_openrouter(&self) -> anyhow::Result<BoxCloneSyncService<ORRequest,ORResponse,anyhow::Error>> {
        match self {
            Self::OpenRouter(client) => Ok(client.get_service()),
            _ => Err(not_an_http_client::<Self>()),
        }
    }

    pub fn get_luaruntime(&self) -> anyhow::Result<BoxCloneSyncService<LuaCall,JSValue,anyhow::Error>> {
        match self {
            Self::LuaRuntime(client) => Ok(client.get_service()),
            _ => Err(not_a_lua_runtime::<Self>()),
        }
    }

    /*
     * This -should- return an HTTP listener
     *
     * I don't know exactly how well this fits into the current model
     * TODO: reconsider/re-work
     *
     */

    pub fn get_endpoint(&self) -> anyhow::Result<Webshit> {

        match self {
            Self::EndPoint(client) => Ok(Webshit::from(client.get_service())),
            _ => Err(not_an_http_server::<Self>()),
        }
    }
}


