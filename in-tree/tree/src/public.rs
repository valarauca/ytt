use std::sync::Arc;
use std::hash::BuildHasherDefault;
use std::collections::HashSet;
use seahash::SeaHasher;
use scc::Shared;
use crate::tree::InternalTree;
use crate::node::RecursiveListing;

#[derive(Clone, Default)]
pub struct Tree<V>
where
    V: 'static + Send + Sync,
{
    inner: Arc<InternalTree<String, V>>,
}

impl<V> Tree<V>
where
    V: 'static + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InternalTree::new()),
        }
    }

    pub fn insert(&self, path: &[&str], value: V) -> Option<Shared<V>> {
        self.inner.insert::<_,str>(path.iter().map(|s| *s), value)
    }

    pub fn get(&self, path: &[&str]) -> Option<Shared<V>> {
        self.inner.get::<_,str>(path.iter().map(|s| *s))
    }

    pub fn contains_path(&self, path: &[&str]) -> bool {
        self.inner.contains_path::<_,str>(path.iter().map(|s| *s))
    }

    pub fn get_or_parent(&self, path: &[&str]) -> Option<Shared<V>> {
        self.inner.get_or_parent::<_,str>(path.iter().map(|s| *s))
    }

    pub fn remove(&self, path: &[&str]) -> Option<Shared<V>> {
        self.inner.remove::<_,str>(path.iter().map(|s| *s))
    }

    pub fn list_children(&self, path: &[&str]) -> Option<HashSet<String, BuildHasherDefault<SeaHasher>>> {
        self.inner.list_children::<_,str>(path.iter().map(|s| *s))
    }

    pub fn list_children_recursive(&self, path: &[&str]) -> Option<RecursiveListing<String>> {
        self.inner.list_children_recursive::<_,str>(path.iter().map(|s| *s))
    }
}
