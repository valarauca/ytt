use std::any::Any;
use crate::{
    traits::{BoxedConfig, not_an_http_client, not_an_http_server},
    adapters::reconfigurable::{Reconfig,ReconfigurableService},
    adapters::s3service::{BoxCloneSyncService},
    services::listenable::{ kinds::{ExtHttpRequest,ExtHttpResponse}, webshit::{Webshit}},
    services::web_request::service_kind::WebClientService,
};
use reqwest::{Request as HttpRequest,Response as HttpResponse};
use openrouter::completions::{Request as ORRequest,Response as ORResponse};


/// Interior service definations
///
/// Handles deligating way the correct request/response
/// type is for an undelrying service
#[non_exhaustive]
pub enum ServiceManagement {
    WebClient(WebClientService),
    OpenRouter(Box<dyn Reconfig<ORRequest,ORResponse> + 'static>),
    EndPoint(Box<dyn Reconfig<ExtHttpRequest,ExtHttpResponse> + 'static>),
}
impl<C> From<ReconfigurableService<C,ExtHttpRequest,ExtHttpResponse>> for ServiceManagement
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
{
    fn from(item: ReconfigurableService<C,ExtHttpRequest,ExtHttpResponse>) -> Self {
        Self::EndPoint(Box::new(item))
    }
}
impl From<WebClientService> for ServiceManagement {
    fn from(item: WebClientService) -> Self {
        Self::WebClient(item)
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
            Self::WebClient(client) => client.reload(config).await,
            Self::OpenRouter(client) => client.reconfig(config).await,
            Self::EndPoint(client) => client.reconfig(config).await,
        }
    }

    /// Get a webclient if this is an instance of one
    pub fn get_web_client(&self) -> Result<BoxCloneSyncService<HttpRequest,HttpResponse,anyhow::Error>,anyhow::Error> {
        match self {
            Self::WebClient(client) => Ok(BoxCloneSyncService::new(client.make_reqwest_service())),
            _ => Err(not_an_http_client::<Self>()),
        }
    }

    pub fn get_openrouter(&self) -> Result<BoxCloneSyncService<ORRequest,ORResponse,anyhow::Error>,anyhow::Error> {
        match self {
            Self::OpenRouter(client) => Ok(client.get_service()),
            _ => Err(not_an_http_client::<Self>()),
        }
    }

    pub fn get_endpoint(&self) -> Result<Webshit,anyhow::Error> {

        match self {
            Self::EndPoint(client) => Ok(Webshit::from(client.get_service())),
            _ => Err(not_an_http_server::<Self>()),
        }
    }
}


