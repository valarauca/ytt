use std::{
    future::{Ready,ready},
    marker::PhantomData,
    task::{Context,Poll},
};
use futures_util::future::{FutureExt};
use tower::{
    Service,
    service_fn,
    util::{ServiceFn},
};
use reloadable::{ReloadingInstance,ReloadableService};
use crate::{
    traits::{Err,BoxedConfig},
    adapters::maybe_async::{MaybeFuture,make_ready},
};
use super::config::{ClientConfig};


/// Uncached Webclient (reqwestclient)
///
/// Acts as an entry point for reloads
pub struct UncachedClient<E: Err> {
    interior: ReloadingInstance<ClientConfig,Factory<E>,ClientCloner<E>>,
}

type Factory<E> = ServiceFn<fn(ClientConfig) -> Ready<Result<ClientCloner<E>,E>>>;

fn factory_impl<E>(config: ClientConfig) -> Ready<Result<ClientCloner<E>,E>>
where
    E: Err + Sized,
{
    let result = config.build()
        .map_err(|e| E::from(e))
        .map(|client: reqwest::Client| -> ClientCloner<E> {
            ClientCloner::<E>::from(client)
        });
    ready(result)
}
impl<E: Err + Sized> UncachedClient<E> {
    pub fn new(config: ClientConfig) -> Result<Self,E> {

        let ptr: fn(ClientConfig) -> Ready<Result<ClientCloner<E>,E>> = factory_impl::<E>;

        // factory returns `Ready` so unwrapping `now_or_never` is safe
        let interior = ReloadingInstance::new(config,service_fn(ptr))
            .now_or_never()
            .unwrap()?;
        Ok(Self { interior } )
    }

    pub fn reload(&mut self, config: BoxedConfig) -> MaybeFuture<Result<(),E>> {
        let config = match config.downcast::<ClientConfig>() {
            Err(_) => return make_ready(Err(E::type_error::<ClientConfig>())),
            Ok(config) => config,
        };
        self.interior.reload(*config)
    }

    pub fn get_service_handle(&self) -> ReloadableService<ClientCloner<E>,reqwest::Client,reqwest::Request> {
        self.interior.get_service_handle::<reqwest::Request,reqwest::Client>()
    }
}


pub struct ClientCloner<E> {
    client: reqwest::Client,
    _marker: PhantomData<fn(E)>,
}
impl<E> From<reqwest::Client> for ClientCloner<E> {
    fn from(client: reqwest::Client) -> ClientCloner<E> {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}
impl<E> Clone for ClientCloner<E> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            _marker: PhantomData,
        }
    }
}
impl<E> Service<()> for ClientCloner<E> {
    type Response = reqwest::Client;
    type Error = E;
    type Future = Ready<Result<reqwest::Client,E>>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),E>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: ()) -> Self::Future {
        ready(Ok(self.client.clone()))
    }
}
