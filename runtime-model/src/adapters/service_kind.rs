
use reloadable::{ReloadableService};

use crate::{
    traits::{Err,BoxedConfig},
    adapters::maybe_async::{MaybeFuture,make_ready},
    services::web_request::service_impl::{UncachedClient,ClientCloner},
};



/// Interior service definations
#[non_exhaustive]
pub enum ServiceManagement<E: Err> {
    WebClient(UncachedClient<E>)
}
impl<E: Err + Sized> ServiceManagement<E> {

    /// Central reloading
    pub fn reload(&mut self, config: BoxedConfig) -> MaybeFuture<Result<(),E>> {
        #[allow(unreachable_patterns)]
        match self {
            Self::WebClient(client) => client.reload(config),
            _ => make_ready(Err(E::type_error::<()>())),
        }
    }

    /// Get a webclient if this is an instance of one
    pub fn get_web_client(&self) -> Result<ReloadableService<ClientCloner<E>,reqwest::Client,reqwest::Request>,E> {
        #[allow(unreachable_patterns)]
        match self {
            Self::WebClient(client) => Ok(client.get_service_handle()),
            _ => Err(E::not_an_http_client::<Self>()),
        }
    }
}

