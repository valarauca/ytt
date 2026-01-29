
use crate::{
    traits::{Err,BoxedConfig},
    adapters::reconfigurable::{Reconfig},
};



/// Interior service definations
///
/// Handles deligating way the correct request/response
/// type is for an undelrying service
#[non_exhaustive]
pub enum ServiceManagement<E: Err> {
    WebClient(Box<dyn Reconfig<reqwest::Request,reqwest::Response,E> + 'static>),
}
impl<E: Err + Sized> ServiceManagement<E> {

    /// Central reloading
    pub async fn reload(&self, config: BoxedConfig) -> Result<(),E> {
        #[allow(unreachable_patterns)]
        match self {
            Self::WebClient(client) => client.reconfig(config).await,
            _ => Err(E::type_error::<()>()),
        }
    }

    /// Get a webclient if this is an instance of one
    pub fn get_web_client(&self) -> Result<tower::util::BoxCloneService<reqwest::Request,reqwest::Response,E>,E> {
        #[allow(unreachable_patterns)]
        match self {
            Self::WebClient(client) => Ok(client.get_service()),
            _ => Err(E::not_an_http_client::<Self>()),
        }
    }
}
