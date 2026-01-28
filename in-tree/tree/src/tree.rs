use std::{
    borrow::{Borrow},
    hash::{Hash,BuildHasherDefault},
    sync::{Arc,atomic::{Ordering}},
    collections::{HashSet},
};
use scc::{
    AtomicShared, Guard, Shared,
    Equivalent,
};
use seahash::SeaHasher;
use sdd::Ptr;
use crate::{
    node::{Node,ensure_not_null,RecursiveListing},
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
    pub(crate) fn insert<'i,I,Q>(self: &Arc<Self>, path: I, value: V) -> Option<Shared<V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ToOwned<Owned=K> + ?Sized + 'i,
    {
        guarded(move |g| {
            let ptr = self.insert_keys_to_location(g, path);
            debug_assert!(!ptr.is_null());
            ptr.as_ref().unwrap().set_here_value(g, value)
        })
    }

    /// Attempts to return item at the exact location
    pub(crate) fn get<'i,I,Q>(self: &Arc<Self>, path: I) -> Option<Shared<V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g| {
            self.walk_to_location(g,path, None)?
                .as_ref()?
                .get_here_value(g)
        })
    }

    pub(crate) fn contains_path<'i,I,Q>(self: &Arc<Self>, path: I) -> bool
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g: &Guard| -> bool {
            self.contains_path_internal(g, path)
        })
    }

    /// Attempts to return item at a specifiction location.
    ///
    /// If path resolution fails, it will attempt to 
    /// if that fails it will try the parent of that location, and parent of that location, etc.
    pub(crate) fn get_or_parent<'i,I,Q>(self: &Arc<Self>, path: I) -> Option<Shared<V>>
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g| -> Option<Shared<V>> {
            let mut parent = Vec::new();
            if let Some(s) = self.walk_to_location(g,path,Some(&mut parent))
                .and_then(|value| value.as_ref())
                .and_then(|node_ref| node_ref.get_here_value(g))
            {
                return Some(s);
            }
            while let Some(ptr) = parent.pop() {
                if let Some(s) = ptr.as_ref()
                    .and_then(|node_ref| node_ref.get_here_value(g))
                {
                    return Some(s);
                }
            }
            None
        })
    }

    /// Returns the item removed on success
    pub(crate) fn remove<'i,I,Q>(self: &Arc<Self>, path: I) -> Option<Shared<V>>
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

    pub(crate) fn list_children<'i,I,Q>(self: &Arc<Self>,path: I) -> Option<HashSet<K,BuildHasherDefault<SeaHasher>>>
    where
        K: Clone + Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g| {
            let ptr = self.walk_to_location(g,path,None)?;
            let node_ref = ptr.as_ref()?;
            Some(node_ref.list_keys())
        })
    }

    pub(crate) fn list_children_recursive<'i,I,Q>(self: &Arc<Self>, path: I) -> Option<RecursiveListing<K>>
    where
        K: Clone + Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        guarded(move |g| {
            let ptr = self.walk_to_location(g,path,None)?;
            let node_ref = ptr.as_ref()?;
            Some(node_ref.list_keys_recursive(g))
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
            let p: Ptr<'g,Node<K,V>> = ptr?;
            if p.is_null() {
                return None;
            }
            if let Some(inner) = &mut stack {
                inner.push(p);
            }
            let node_ref: &'g Node<K,V> = p.as_ref().unwrap();
            ptr = node_ref.get_child_node_ptr(guard, curr_key);
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

    fn contains_path_internal<'g,'i,I,Q>(&'g self, g: &'g Guard, path: I) -> bool
    where
        K: Borrow<Q>,
        I: IntoIterator<Item=&'i Q> + 'i,
        Q: Eq + Equivalent<K> + Hash + ?Sized + 'i,
    {
        let ptr: Option<Ptr<'g, Node<K,V>>> = Some(self.root.load(Ordering::Acquire,g));
        let (exists, _ ) = path.into_iter()
            .fold((true,ptr), |(ok,ptr): (bool, Option<Ptr<'g,Node<K,V>>>), key: &'i Q| -> (bool,Option<Ptr<'g,Node<K,V>>>) {
                if !ok {
                    return (false, None);
                }
                let node_ref = match ptr.and_then(|p| p.as_ref()) {
                    None => return (false, None),
                    Some(node_ref) => node_ref,
                };
                if !node_ref.has_child(key) {
                    return (false, None);
                }
                (true, node_ref.get_child_node_ptr(g,key))
            });
        exists
    }
}


#[test]
fn test_insert() {
    let tree = Arc::new(InternalTree::<String,usize>::default());
    assert!(tree.insert::<_,str>([], 1usize).is_none());
    assert!(tree.insert::<_,str>(["foo","bar"], 3usize).is_none());
    assert!(tree.insert::<_,str>(["foo"], 2usize).is_none());
    assert!(tree.insert::<_,str>(["foo", "bar", "baz"], 4usize).is_none());

    assert_eq!(*tree.get::<_,str>([]).unwrap(),1usize);
    assert_eq!(*tree.get::<_,str>(["foo"]).unwrap(), 2usize);
    assert_eq!(*tree.get::<_,str>(["foo","bar"]).unwrap(), 3usize);
    assert_eq!(*tree.get::<_,str>(["foo","bar", "baz"]).unwrap(), 4usize);
}

#[test]
fn test_get_or_parent() {
    let tree = Arc::new(InternalTree::<String,usize>::default());
    assert!(tree.insert::<_,str>([], 1usize).is_none());
    assert!(tree.insert::<_,str>(["foo","bar"], 3usize).is_none());
    assert!(tree.insert::<_,str>(["foo", "bar", "baz", "foobar"], 5usize).is_none());

    assert_eq!(*tree.get_or_parent::<_,str>(["foo"]).unwrap(), 1usize);
    assert_eq!(*tree.get_or_parent::<_,str>(["foo","bar","baz"]).unwrap(), 3usize);
    assert_eq!(*tree.get_or_parent::<_,str>([]).unwrap(),1usize);
    assert_eq!(*tree.get_or_parent::<_,str>(["foo","bar"]).unwrap(), 3usize);
    assert_eq!(*tree.get_or_parent::<_,str>(["foo","bar", "baz", "foobar"]).unwrap(), 5usize);
}

#[test]
fn test_new() {
    let tree = Arc::new(InternalTree::<String,usize>::new());
    assert!(tree.get::<_,str>([]).is_none());
}

#[test]
fn test_contains_path() {
    let tree = Arc::new(InternalTree::<String,usize>::default());
    assert!(tree.contains_path::<_,str>([]));
    assert!(!tree.contains_path::<_,str>(["foo"]));

    assert!(tree.insert::<_,str>([], 1usize).is_none());
    assert!(tree.contains_path::<_,str>([]));

    assert!(tree.insert::<_,str>(["foo","bar"], 2usize).is_none());
    assert!(tree.contains_path::<_,str>(["foo"]));
    assert!(tree.contains_path::<_,str>(["foo","bar"]));
    assert!(!tree.contains_path::<_,str>(["foo","baz"]));
}

#[test]
fn test_remove() {
    let tree = Arc::new(InternalTree::<String,usize>::default());
    assert!(tree.insert::<_,str>([], 1usize).is_none());
    assert!(tree.insert::<_,str>(["foo"], 2usize).is_none());
    assert!(tree.insert::<_,str>(["foo","bar"], 3usize).is_none());

    assert_eq!(*tree.remove::<_,str>(["foo","bar"]).unwrap(), 3usize);
    assert!(tree.get::<_,str>(["foo","bar"]).is_none());
    assert_eq!(*tree.get::<_,str>(["foo"]).unwrap(), 2usize);
    assert_eq!(*tree.get::<_,str>([]).unwrap(), 1usize);

    assert_eq!(*tree.remove::<_,str>(["foo"]).unwrap(), 2usize);
    assert!(tree.get::<_,str>(["foo"]).is_none());

    assert!(tree.remove::<_,str>(["nonexistent"]).is_none());
}

#[test]
fn test_list_children() {
    let tree = Arc::new(InternalTree::<String,usize>::default());
    assert!(tree.insert::<_,str>([], 1usize).is_none());
    assert!(tree.insert::<_,str>(["foo"], 2usize).is_none());
    assert!(tree.insert::<_,str>(["bar"], 3usize).is_none());
    assert!(tree.insert::<_,str>(["baz"], 4usize).is_none());

    let children = tree.list_children::<_,str>([]).unwrap();
    assert_eq!(children.len(), 3);
    assert!(children.contains("foo"));
    assert!(children.contains("bar"));
    assert!(children.contains("baz"));

    let children = tree.list_children::<_,str>(["foo"]).unwrap();
    assert_eq!(children.len(), 0);

    assert!(tree.list_children::<_,str>(["nonexistent"]).is_none());
}

#[test]
fn test_list_children_recursive() {
    let tree = Arc::new(InternalTree::<String,usize>::default());
    assert!(tree.insert::<_,str>([], 1usize).is_none());
    assert!(tree.insert::<_,str>(["foo"], 2usize).is_none());
    assert!(tree.insert::<_,str>(["foo","bar"], 3usize).is_none());
    assert!(tree.insert::<_,str>(["foo","bar","baz"], 4usize).is_none());
    assert!(tree.insert::<_,str>(["qux"], 5usize).is_none());

    let listing = tree.list_children_recursive::<_,str>([]).unwrap();
    assert!(listing.children.contains_key("foo"));
    assert!(listing.children.contains_key("qux"));
    assert!(listing.children["foo"].children.contains_key("bar"));
    assert!(listing.children["foo"].children["bar"].children.contains_key("baz"));

    let listing = tree.list_children_recursive::<_,str>(["foo"]).unwrap();
    assert!(listing.children.contains_key("bar"));
    assert!(listing.children["bar"].children.contains_key("baz"));

    assert!(tree.list_children_recursive::<_,str>(["nonexistent"]).is_none());
}
