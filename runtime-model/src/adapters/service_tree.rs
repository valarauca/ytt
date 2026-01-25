
use std::{
    sync::Arc,
};
use tokio::sync::RwLock;

use tree::Tree;

use crate::{
    traits::{RegisteredService,Err,BoxedConfig,ServiceObj},
};

/// Top level system for service management
pub struct RegisteredServiceTree<E: Err> {
    inner: Tree<Arc<RwLock<Box<dyn RegisteredService<E>>>>>,
}
impl<E: Err> Clone for RegisteredServiceTree<E> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
impl<E: Err> Default for RegisteredServiceTree<E> {
    fn default() -> Self {
        Self { inner: Tree::default() }
    }
}
impl<E: Err> RegisteredServiceTree<E> {

    /// Trigger a service reload.
    pub async fn reload(&self, path: &[&str], config: BoxedConfig) -> Result<(),E>
    where
        E: Sized,
    {
        let service = match self.inner.get(path) {
            None => return Err(E::no_such_service(path)),
            Some(x) => {
                // type infernece is funky here
                let s: Arc<RwLock<Box<dyn RegisteredService<E>>>> = (*x).clone();
                s
            }
        };
        service.write_owned().await.reload(config)?.await
    }

    pub async fn get_service<Req,Res>(&self, path: &[&str], func: fn(&dyn RegisteredService<E>) -> Result<ServiceObj<Req,Res,E>,E>) -> Result<ServiceObj<Req,Res,E>,E>
    where
        Req: 'static + Send,
        Res: 'static + Send,
    {
        let service = match self.inner.get(path) {
            None => return Err(E::no_such_service(path)),
            Some(x) => {
                let s: Arc<RwLock<Box<dyn RegisteredService<E>>>> = (*x).clone();
                s
            }
        };
        let r = service.read_owned().await;
        (func)(r.as_ref())
    }

    pub async fn get_or_parent_service<Req,Res>(&self, path: &[&str], func: fn(&dyn RegisteredService<E>) -> Result<ServiceObj<Req,Res,E>,E>) -> Result<ServiceObj<Req,Res,E>,E>
    where
        Req: 'static + Send,
        Res: 'static + Send,
    {
        let service = match self.inner.get_or_parent(path) {
            None => return Err(E::no_such_service(path)),
            Some(x) => {
                let s: Arc<RwLock<Box<dyn RegisteredService<E>>>> = (*x).clone();
                s
            }
        };
        let r = service.read_owned().await;
        (func)(r.as_ref())
    }

}
