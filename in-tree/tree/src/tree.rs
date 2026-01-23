use std::{
    borrow::{Borrow},
    hash::{Hash,BuildHasherDefault},
    sync::{Arc,atomic::{Ordering}},
};
use scc::{
    AtomicShared, Guard, HashSet, Shared, Tag,
    hash_index::{Entry, HashIndex},
    Equivalent,
};
use sdd::Ptr;
use crate::{
    node::{Node,ensure_not_null},
    guarded::{guarded},
};

pub(crate) struct InternalTree<K,V>
where
    K: 'static + Send + Sync + Hash + Eq,
    V: 'static + Send + Sync,
{
    root: AtomicShared<Node<K,V>>,
}
impl<K,V> Default for InternalTree<K,V>
where
    K: 'static + Send + Sync + Hash + Eq,
    V: 'static + Send + Sync,
{
    fn default() -> Self {
        Self { root: AtomicShared::null() }
    }
}
impl<K,V> InternalTree<K,V>
where
    K: 'static + Send + Sync + Hash + Eq,
    V: 'static + Send + Sync,
{
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Insert a value at a given path, will return the old value if one existed.
    pub(crate) fn insert<'i,I,Q>(self: Arc<Self>, path: I, value: V) -> Option<Shared<V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ToOwned<Owned=K> + ?Sized + 'i,
    {
        guarded(move |g| {
            let ptr = self.insert_keys_to_location(&g, path);
            debug_assert!(!ptr.is_null());
            ptr.as_ref().unwrap().set_here_value(&g, value)
        })
    }

    /// Attempts to return item at the exact location
    pub(crate) fn get<'i,I,Q>(self: Arc<Self>, path: I) -> Option<Shared<V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g| {
            let ptr = self.walk_to_location(g,path, None)?;
            let node_ref = ptr.as_ref()?;
            let here = node_ref.here.load(Ordering::Acquire,g);
            here.get_shared()
        })
    }

    /// Attempts to return item at a specifiction location.
    ///
    /// If path resolution fails, it will attempt to 
    /// if that fails it will try the parent of that location, and parent of that location, etc.
    pub(crate) fn get_or_parent<'i,I,Q>(self: Arc<Self>, path: I) -> Option<Shared<V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g| -> Option<Shared<V>> {
            let mut parent = Vec::new();
            if let Some(s) = self.walk_to_location(g,path,Some(&mut parent))
                .map(|value| value.as_ref())
                .flatten()
                .map(|node_ref| node_ref.here.load(Ordering::Acquire,g).get_shared())
                .flatten()
            {
                return Some(s);
            }
            while let Some(ptr) = parent.pop() {
                if let Some(s) = ptr.as_ref()
                    .map(|node_ref| node_ref.here.load(Ordering::Acquire,g).get_shared())
                    .flatten()
                {
                    return Some(s);
                }
            }
            None
        })
    }

    /// Returns the item removed on success
    pub(crate) fn remove<'i,I,Q>(self: Arc<Self>, path: I) -> Option<Shared<V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g| {
            let ptr = self.walk_to_location(g,path,None)?;
            let node_ref = ptr.as_ref()?;
            node_ref.remove_here_value(g)
        })
    }

    /*
     * Internal Helper Methods
     *
     */

    fn walk_to_location<'g,'i, I, Q>(&'g self, guard: &'g Guard, path: I, mut stack: Option<&mut Vec<Ptr<'g,Node<K,V>>>>) -> Option<Ptr<'g,Node<K,V>>>
    where
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
        K: Borrow<Q>,
    {
        let mut ptr: Option<Ptr<'g, Node<K,V>>> = Some(self.root.load(Ordering::Acquire,guard));
        for curr_key in path.into_iter() {
            let ptr: Ptr<'g,Node<K,V>> = match ptr {
                None => return None,
                Some(x) => x,
            };
            if ptr.is_null() {
                return None;
            }
            if let Some(inner) = &mut stack {
                inner.push(ptr);
            }
            let node_ref: &'g Node<K,V> = ptr.as_ref().unwrap();
            let ptr = node_ref.get_child_node_ptr(guard, curr_key);
        }
        ptr
    }

    fn insert_keys_to_location<'g,'i, I, Q>(&'g self, g: &'g Guard, path: I) -> Ptr<'g,Node<K,V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ToOwned<Owned=K> + ?Sized + 'i,
    {
        let mut ptr: Ptr<'g, Node<K,V>> = ensure_not_null(&self.root, g);
        for curr_key in path.into_iter() {
            debug_assert!(!ptr.is_null());
            let node_ref = ptr.as_ref().unwrap();
            ptr = node_ref.upsert_child_key(g, curr_key);
        }
        ptr
    }
}


