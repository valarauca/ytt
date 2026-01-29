
use std::{
    sync::Arc,
    hash::BuildHasherDefault,
    collections::HashSet,
};
use tokio::sync::{RwLock};
use seahash::SeaHasher;
use tree::{Tree,RecursiveListing};

use crate::traits::{Err,BoxedConfig};
use super::maybe_async::{MaybeFuture,MaybeErrAccess,make_boxed,MutexGuard,make_ready};
use super::{ServiceManagement};



#[allow(type_alias_bounds)]
pub type ServiceFetch<O,E: Err> = fn(&ServiceManagement<E>) -> Result<O,E>;

#[allow(type_alias_bounds)]
type ManagementGuard<E: Err> = Arc<RwLock<ServiceManagement<E>>>;

/// Top level system for service management
pub struct RegisteredServiceTree<E: Err> {
    inner: Tree<Arc<RwLock<ServiceManagement<E>>>>,
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

    pub fn get_service_exact<O>(&self, path: &[&str], func: ServiceFetch<O,E>) -> MaybeFuture<Result<O,E>>
    where
        O: Send + 'static,
        ServiceFetch<O,E>: Send + 'static,
    {
        self.inner
            .get(path)
            .map(|s| -> ManagementGuard<E> { (*s).clone() })
            .ok_or_else(|| E::no_such_service(path))
            .do_read::<_,O>(func)
    }

    pub fn get_service<O>(&self, path: &[&str], func: ServiceFetch<O,E>) -> MaybeFuture<Result<O,E>>
    where
        O: Send + 'static,
        ServiceFetch<O,E>: Send + 'static,
    {
        self.inner
            .get_or_parent(path)
            .map(|s| -> ManagementGuard<E> { (*s).clone() })
            .ok_or_else(|| E::no_such_service(path))
            .do_read::<_,O>(func)
    }

    /// Trigger a service reload.
    pub fn reload(&self, path: &[&str], config: BoxedConfig) -> MaybeFuture<Result<(),E>>
    where
        E: Sized,
    {
        let arc = match self.inner
            .get(path)
            .map(|s| -> ManagementGuard<E> { (*s).clone() })
            .ok_or_else(|| E::no_such_service(path))
        {
            Err(e) => return make_ready(Err(e)),
            Ok(x) => x,
        };
        match arc.sync_read() {
            Ok(guard) => {
                make_boxed(async move {
                    guard.reload(config).await
                })
            }
            Err(arc) => {
                make_boxed(async move {
                    let arc: ManagementGuard<E> = arc;
                    let guard = arc.async_write().await;
                    guard.reload(config).await
                })
            }
        }
    }

    pub fn insert(&self, path: &[&str], item: ServiceManagement<E>) -> bool {
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
