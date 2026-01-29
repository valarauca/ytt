
use reloadable::{ReloadableService};

use crate::{
    traits::{Err,BoxedConfig},
    adapters::maybe_async::{MaybeFuture,make_ready},
    adapters::reconfigurable::{ReconfigurableService,RequestHandle},
    services::web_request::config::{ClientConfig},
};



/// Interior service definations
#[non_exhaustive]
pub enum ServiceManagement<E: Err> {
    WebClient(ReconfigurableService<ClientConfig,reqwest::Request,reqwest::Response,E>),
}
impl<E: Err + Sized> ServiceManagement<E> {

    /// Central reloading
    pub async fn reload(&self, config: BoxedConfig) -> Result<(),E> {
        #[allow(unreachable_patterns)]
        match self {
            Self::WebClient(client) => {
                let x: Box<ClientConfig> = config.downcast().map_err(|_| E::type_error::<ClientConfig>())?;
                client.reconfigure(*x).await
            }
            _ => Err(E::type_error::<()>()),
        }
    }

    /// Get a webclient if this is an instance of one
    pub fn get_web_client(&self) -> Result<RequestHandle<ClientConfig,reqwest::Request,reqwest::Response,E>,E> {
        #[allow(unreachable_patterns)]
        match self {
            Self::WebClient(client) => Ok(client.make_request_handle()),
            _ => Err(E::not_an_http_client::<Self>()),
        }
    }
}

