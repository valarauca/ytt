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
use super::{ServiceManagement,GetTreePath};


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

    pub fn get_service_exact<O,P>(&self, path: &P, func: ServiceFetch<O>) -> MaybeFuture<Result<O,anyhow::Error>>
    where
        P: GetTreePath + ?Sized,
        O: Send + 'static,
        ServiceFetch<O>: Send + 'static,
    {
        let path = match path.get_tree_path() {
            Ok(x) => x,
            Err(e) => {
                return make_ready(Err(e));
            }
        };
        self.inner
            .get(&path)
            .map(|s| -> ManagementGuard { (*s).clone() })
            .ok_or_else(|| no_such_service(&path))
            .do_read::<_,O>(func)
    }

    pub fn get_service<O,P>(&self, path: &P, func: ServiceFetch<O>) -> MaybeFuture<Result<O,anyhow::Error>>
    where
        P: GetTreePath + ?Sized,
        O: Send + 'static,
        ServiceFetch<O>: Send + 'static,
    {
        let path = match path.get_tree_path() {
            Ok(x) => x,
            Err(e) => {
                return make_ready(Err(e));
            }
        };
        self.inner
            .get_or_parent(&path)
            .map(|s| -> ManagementGuard { (*s).clone() })
            .ok_or_else(|| no_such_service(&path))
            .do_read::<_,O>(func)
    }

    /// Trigger a service reload.
    pub fn reload<P>(&self, path: &P, config: BoxedConfig) -> MaybeFuture<Result<(),anyhow::Error>>
    where
        P: GetTreePath + ?Sized,
    {
        let path = match path.get_tree_path() {
            Ok(x) => x,
            Err(e) => {
                return make_ready(Err(e));
            }
        };
        let arc = match self.inner
            .get(&path)
            .map(|s| -> ManagementGuard { (*s).clone() })
            .ok_or_else(|| no_such_service(&path))
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

    pub fn insert<P>(&self, path: &P, item: ServiceManagement) -> anyhow::Result<bool> 
    where
        P: GetTreePath + ?Sized,
    {
        let path = path.get_tree_path()?;
        Ok(self.inner.insert(&path, Arc::new(RwLock::new(item))).is_some())
    }

    pub fn remove<P>(&self, path: &P) -> Result<(),anyhow::Error>
    where
        P: GetTreePath + ?Sized,
    {
        let path = path.get_tree_path()?;
        self.inner.remove(&path).map(|_| ()).ok_or_else(|| no_such_service(&path))
    }

    pub fn contains_path<P>(&self, path: &P) -> Result<bool,anyhow::Error>
    where
        P: GetTreePath + ?Sized,
    {
        let path = path.get_tree_path()?;
        Ok(self.inner.contains_path(&path))
    }

    pub fn list_children<P>(&self, path: &P) -> anyhow::Result<Option<HashSet<String, BuildHasherDefault<SeaHasher>>>>
    where
        P: GetTreePath + ?Sized,
    {
        let path = path.get_tree_path()?;
        Ok(self.inner.list_children(&path))
    }

    pub fn list_children_recursive<P>(&self, path: &P) -> anyhow::Result<Option<RecursiveListing<String>>>
    where
        P: GetTreePath + ?Sized,
    {
        let path = path.get_tree_path()?;
        Ok(self.inner.list_children_recursive(&path))
    }
}
