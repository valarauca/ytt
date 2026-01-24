
use std::{
    sync::Arc,
    pin::Pin,
    future::Future,
};

use tokio::{
    sync::RwLock,
};

use tree::Tree;

use crate::{
    traits::{RegisteredService,Err,BoxedConfig},
};

/// Top level system for service management
#[derive(Clone)]
pub struct RegisteredServiceTree<E: Err> {
    inner: Tree<Arc<RwLock<Box<dyn RegisteredService<E>>>>>,
}
impl<E: Err> Default for RegisteredServiceTree<E> {
    fn default() -> Self {
        Self { inner: Tree::default() }
    }
}
impl<E: Err> RegisteredServiceTree<E> {

    /// Trigger a service reload.
    pub fn reload(&self, path: &[&str], config: BoxedConfig) -> Result<Pin<Box<dyn Future<Output=Result<(),E>> + 'static + Send>>,E>
    where
        E: Sized,
    {
        let service = match self.inner.get(path) {
            None => return Err(E::no_such_service(path)),
            Some(x) => {
                // use government name so type shit works
                let s: Arc<RwLock<Box<dyn RegisteredService<E>>>> = (*x).clone();
                s
            }
        };
        Ok(Box::pin(async move { 
            service.write_owned().await.reload(config)?.await
        }))
    }
}
