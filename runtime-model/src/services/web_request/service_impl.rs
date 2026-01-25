use std::{
    future::{ready,Ready,Future},
    marker::PhantomData,
    pin::Pin,
};

use futures_util::{
    future::{FutureExt,TryFutureExt},
};
use reqwest::{
    Client,
    Error as ReqwestError,
    Request as ReqwestRequest,
    Response as ReqwestResponse,
};
use tower::{
    Service,ServiceExt,ServiceBuilder,
    util::{ServiceFn},
};
use reloadable::{ReloadableService,ReloadingInstance};

use super::config::{ClientConfig};
use crate::traits::{Err,RegisteredService,ServiceKind,BoxedConfig,ServiceObj};


pub struct ReqwestWrapper<E> {
    interior: ReloadingInstance<ClientConfig,Client,ServiceFn<fn(ClientConfig) -> Ready<Result<Client,ReqwestError>>>>,
    _marker: PhantomData<fn(E)>,
}
impl<E: Err> ReqwestWrapper<E> {
    fn get_service(&self) -> impl Service<ReqwestRequest,Response=ReqwestResponse,Error=reloadable::ReloadableServiceError<reqwest::Error>,Future: Send + 'static> + 'static {
        self.interior.service::<ReqwestRequest>()
    }
}
impl<E: Err> RegisteredService<E> for ReqwestWrapper<E> {

    fn get_priority(&self) -> usize { 10 }

    fn get_roles(&self) -> &'static [ServiceKind] { &[ServiceKind::HttpClient] }

    fn reload<'a>(&'a mut self, config: BoxedConfig) -> Result<Pin<Box<dyn Future<Output=Result<(),E>> + 'a + Send>>,E> 
    where
        E: Sized,
    {
        let x = config.downcast::<ClientConfig>().map_err(|_| E::type_error::<ClientConfig>())?;
        Ok(Box::pin(async move {
            self.interior.ready().await?;

            self.interior.call(*x).await?;
            Ok(())
        }))
    }

    /// Return a handle to an http client
    fn get_http_client(&self) -> Result<ServiceObj<ReqwestRequest,ReqwestResponse,E,E>,E>
    where
        E: Sized,
    {
        let s = self.get_service()
            .map_future(|f| f.map_err(|e| E::from(e)).boxed());
        Ok(Box::new(s))
    }
}
