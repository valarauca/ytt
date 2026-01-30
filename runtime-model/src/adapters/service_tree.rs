use std::{
    sync::{Arc,LazyLock},
    hash::BuildHasherDefault,
    collections::HashSet,
};
use tokio::sync::RwLock;
use seahash::SeaHasher;
use tree::{Tree,RecursiveListing};

use crate::traits::{BoxedConfig, no_such_service};
use super::maybe_async::{MaybeFuture,MaybeErrAccess,MutexGuard,make_boxed,make_ready};
use super::ServiceManagement;


static GLOBAL_TREE: LazyLock<RegisteredServiceTree> = LazyLock::new(|| RegisteredServiceTree::default());

pub fn get_tree() -> RegisteredServiceTree {
    (*GLOBAL_TREE).clone()
}

pub type ServiceFetch<O> = fn(&ServiceManagement) -> Result<O,anyhow::Error>;

type ManagementGuard = Arc<RwLock<ServiceManagement>>;



/// Top level system for service management
pub struct RegisteredServiceTree {
    inner: Tree<Arc<RwLock<ServiceManagement>>>,
}
impl Clone for RegisteredServiceTree {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
impl Default for RegisteredServiceTree {
    fn default() -> Self {
        Self { inner: Tree::default() }
    }
}
impl RegisteredServiceTree {

    pub fn get_service_exact<O>(&self, path: &[&str], func: ServiceFetch<O>) -> MaybeFuture<Result<O,anyhow::Error>>
    where
        O: Send + 'static,
        ServiceFetch<O>: Send + 'static,
    {
        self.inner
            .get(path)
            .map(|s| -> ManagementGuard { (*s).clone() })
            .ok_or_else(|| no_such_service(path))
            .do_read::<_,O>(func)
    }

    pub fn get_service<O>(&self, path: &[&str], func: ServiceFetch<O>) -> MaybeFuture<Result<O,anyhow::Error>>
    where
        O: Send + 'static,
        ServiceFetch<O>: Send + 'static,
    {
        self.inner
            .get_or_parent(path)
            .map(|s| -> ManagementGuard { (*s).clone() })
            .ok_or_else(|| no_such_service(path))
            .do_read::<_,O>(func)
    }

    /// Trigger a service reload.
    pub fn reload(&self, path: &[&str], config: BoxedConfig) -> MaybeFuture<Result<(),anyhow::Error>>
    {
        let arc = match self.inner
            .get(path)
            .map(|s| -> ManagementGuard { (*s).clone() })
            .ok_or_else(|| no_such_service(path))
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
                    let arc: ManagementGuard = arc;
                    let guard = arc.async_write().await;
                    guard.reload(config).await
                })
            }
        }
    }

    pub fn insert(&self, path: &[&str], item: ServiceManagement) -> bool {
        self.inner.insert(path, Arc::new(RwLock::new(item))).is_some()
    }

    pub fn remove(&self, path: &[&str]) -> Result<(),anyhow::Error> {
        self.inner.remove(path).map(|_| ()).ok_or_else(|| no_such_service(path))
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
