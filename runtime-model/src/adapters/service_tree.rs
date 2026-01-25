
use std::{
    sync::Arc,
    hash::BuildHasherDefault,
    collections::HashSet,
    future::Future,
};
use tokio::sync::RwLock;
use futures_util::{
    future::{Either},
};
use seahash::SeaHasher;
use tree::{Tree,RecursiveListing};

use crate::{
    traits::{RegisteredService,Err,BoxedConfig,ServiceObj},
};

use super::maybe_async::{MaybeFuture,MaybeSyncAccess,MaybeErrAccess};


pub type ServiceFetch<Req,Res,E> = fn(&BoxedService<E>) -> Result<ServiceObj<Req,Res,E>,E>;
pub type BoxedService<E> = Box<dyn RegisteredService<E> + 'static + Send + Sync>;
type ServiceGuard<E> = Arc<RwLock<BoxedService<E>>>;


/// Top level system for service management
pub struct RegisteredServiceTree<E: Err> {
    inner: Tree<Arc<RwLock<BoxedService<E>>>>,
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
                let s: ServiceGuard<E> = (*x).clone();
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
                let s: ServiceGuard<E> = (*x).clone();
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
                let s: ServiceGuard<E> = (*x).clone();
                s
            }
        };
        let r = service.read_owned().await;
        (func)(r.as_ref())
    }

    pub fn get_service_exact<Req,Res>(&self, path: &[&str], func: ServiceFetch<Req,Res,E>) -> MaybeFuture<Result<ServiceObj<Req,Res,E>,E>>
    where
        Req: Send + 'static,
        Res: Send + 'static,
        ServiceFetch<Req,Res,E>: Send + 'static,
    {

        self.inner
            .get(path)
            .map(|s| -> ServiceGuard<E> { (*s).clone() })
            .ok_or_else(|| E::no_such_service(path))
            .do_read::<_,ServiceObj<Req,Res,E>>(func)
    }

    pub fn contains_service(&self) -> MaybeFuture<bool> {
        todo!()
    }

    pub async fn remote_service(&self) -> bool {
        todo!()
    }

    pub fn list_children(&self, path: &[&str]) -> Option<HashSet<String, BuildHasherDefault<SeaHasher>>> {
        todo!()
    }

    pub async fn list_children_recursive(&self, path: &[&str]) -> Result<RecursiveListing<String>,E> {
        todo!()
    }

}
