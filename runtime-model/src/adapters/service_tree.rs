
use std::{
    sync::Arc,
    hash::BuildHasherDefault,
    collections::HashSet,
    future::{Future,ready},
};
use tokio::sync::{
    RwLock,
    OwnedRwLockWriteGuard,
};
use futures_util::{
    future::{Either,FutureExt},
};
use seahash::SeaHasher;
use tree::{Tree,RecursiveListing};

use crate::{
    traits::{RegisteredService,Err,BoxedConfig,ServiceObj},
};

use super::maybe_async::{MaybeFuture,MaybeSyncAccess,MaybeErrAccess,make_boxed,MutexGuard};


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

    pub fn get_service<Req,Res>(&self, path: &[&str], func: ServiceFetch<Req,Res,E>) -> MaybeFuture<Result<ServiceObj<Req,Res,E>,E>>
    where
        Req: Send + 'static,
        Res: Send + 'static,
        ServiceFetch<Req,Res,E>: Send + 'static,
    {
        self.inner
            .get_or_parent(path)
            .map(|s| -> ServiceGuard<E> { (*s).clone() })
            .ok_or_else(|| E::no_such_service(path))
            .do_read::<_,ServiceObj<Req,Res,E>>(func)
    }

    /// Trigger a service reload.
    pub fn reload(&self, path: &[&str], config: BoxedConfig) -> MaybeFuture<Result<(),E>>
    where
        E: Sized,
    {
        let arc = match self.inner
            .get(path)
            .map(|s| -> ServiceGuard<E> { (*s).clone() })
            .ok_or_else(|| E::no_such_service(path))
        {
            Err(e) => return ready(Err(e)).left_future(),
            Ok(x) => x,
        };
        match arc.sync_write() {
            Ok(mut guard) => {
                make_boxed(async move { guard.reload(config)?.await })
            }
            Err(arc) => {
                make_boxed(async move {
                    let arc: ServiceGuard<E> = arc;
                    let mut guard = arc.async_write().await;
                    guard.reload(config)?.await
                })
            }
        }
    }

    pub fn insert(&self, path: &[&str], item: BoxedService<E>) -> bool {
        self.inner.insert(path, Arc::new(RwLock::new(item))).is_some()
    }

    pub fn remove(&self, path: &[&str]) -> Result<(),E> {
        self.inner.remove(path).map(|_| ()).ok_or_else(|| E::no_such_service(path))
    }

    pub fn contains_path(&self,path: &[&str]) -> bool {
        self.inner.contains_path(path)
    }

    pub fn list_children(&self, path: &[&str]) -> Option<HashSet<String, BuildHasherDefault<SeaHasher>>> {
        self.inner.list_children(path)
    }

    pub fn list_children_recursive(&self, path: &[&str]) -> Option<RecursiveListing<String>> {
        self.inner.list_children_recursive(path)
    }
}
