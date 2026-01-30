use std::{
    any::Any,
};
use crate::{
    traits::{Err,BoxedConfig},
    adapters::reconfigurable::{Reconfig,ReconfigurableService},
};
use reqwest::{Request as HttpRequest,Response as HttpResponse};
use openrouter::completions::{Request as ORRequest,Response as ORResponse};


/// Interior service definations
///
/// Handles deligating way the correct request/response
/// type is for an undelrying service
#[non_exhaustive]
pub enum ServiceManagement<E: Err> {
    WebClient(Box<dyn Reconfig<HttpRequest,HttpResponse,E> + 'static>),
    OpenRouter(Box<dyn Reconfig<ORRequest,ORResponse,E> + 'static>),
}
impl<C,E: Err> From<ReconfigurableService<C,HttpRequest,HttpResponse,E>> for ServiceManagement<E>
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,HttpRequest,HttpResponse,E>) -> Self {
        Self::WebClient(Box::new(item))
    }
}
impl<C,E: Err> From<ReconfigurableService<C,ORRequest,ORResponse,E>> for ServiceManagement<E>
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,ORRequest,ORResponse,E>) -> Self {
        Self::OpenRouter(Box::new(item))
    }
}
impl<E: Err> ServiceManagement<E> {

    /// Central reloading
    pub async fn reload(&self, config: BoxedConfig) -> Result<(),E> {
        match self {
            Self::WebClient(client) => client.reconfig(config).await,
            Self::OpenRouter(client) => client.reconfig(config).await,
        }
    }

    /// Get a webclient if this is an instance of one
    pub fn get_web_client(&self) -> Result<tower::util::BoxCloneService<HttpRequest,HttpResponse,E>,E> {
        match self {
            Self::WebClient(client) => Ok(client.get_service()),
            _ => Err(E::not_an_http_client::<Self>()),
        }
    }

    pub fn get_openrouter(&self) -> Result<tower::util::BoxCloneService<ORRequest,ORResponse,E>,E> {
        match self {
            Self::OpenRouter(client) => Ok(client.get_service()),
            _ => Err(E::not_an_http_client::<Self>()),
        }
    }
}
