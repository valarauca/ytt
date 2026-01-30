use std::any::Any;
use crate::{
    traits::{BoxedConfig, not_an_http_client},
    adapters::reconfigurable::{Reconfig,ReconfigurableService},
};
use reqwest::{Request as HttpRequest,Response as HttpResponse};
use openrouter::completions::{Request as ORRequest,Response as ORResponse};


/// Interior service definations
///
/// Handles deligating way the correct request/response
/// type is for an undelrying service
#[non_exhaustive]
pub enum ServiceManagement {
    WebClient(Box<dyn Reconfig<HttpRequest,HttpResponse> + 'static>),
    OpenRouter(Box<dyn Reconfig<ORRequest,ORResponse> + 'static>),
}
impl<C> From<ReconfigurableService<C,HttpRequest,HttpResponse>> for ServiceManagement
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,HttpRequest,HttpResponse>) -> Self {
        Self::WebClient(Box::new(item))
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
impl ServiceManagement {

    /// Central reloading
    pub async fn reload(&self, config: BoxedConfig) -> Result<(),anyhow::Error> {
        match self {
            Self::WebClient(client) => client.reconfig(config).await,
            Self::OpenRouter(client) => client.reconfig(config).await,
        }
    }

    /// Get a webclient if this is an instance of one
    pub fn get_web_client(&self) -> Result<tower::util::BoxCloneService<HttpRequest,HttpResponse,anyhow::Error>,anyhow::Error> {
        match self {
            Self::WebClient(client) => Ok(client.get_service()),
            _ => Err(not_an_http_client::<Self>()),
        }
    }

    pub fn get_openrouter(&self) -> Result<tower::util::BoxCloneService<ORRequest,ORResponse,anyhow::Error>,anyhow::Error> {
        match self {
            Self::OpenRouter(client) => Ok(client.get_service()),
            _ => Err(not_an_http_client::<Self>()),
        }
    }
}
