use std::{
    borrow::{Borrow},
    hash::{Hash,BuildHasherDefault},
    sync::atomic::{Ordering},
    collections::{HashSet,HashMap},
};
use scc::{
    AtomicShared, Guard, Shared, Tag,
    hash_index::{Entry, HashIndex},
    Equivalent,
};
use sdd::Ptr;
use seahash::SeaHasher;


pub(crate) struct Node<K,V>
where
    K: 'static + Send + Sync + Hash + Eq,
    V: 'static + Send + Sync,
{
    pub(crate) children: HashIndex<K, AtomicShared<Node<K,V>>, BuildHasherDefault<SeaHasher>>,
    pub(crate) here: AtomicShared<V>,
}
impl<K,V> Default for Node<K,V>
where
    K: 'static + Send + Sync + Hash + Eq,
    V: 'static + Send + Sync,
{
    fn default() -> Self {
        Self {
            children: HashIndex::default(),
            here: AtomicShared::null(),
        }
    }
}
impl<K,V> Node<K,V>
where
    K: 'static + Send + Sync + Hash + Eq,
    V: 'static + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            children: HashIndex::default(),
            here: AtomicShared::null(),
        }
    }
   
    pub(crate) fn get_child_node_ptr<'g, Q>(&self, g: &'g Guard, key: &Q) -> Option<Ptr<'g,Node<K,V>>>
    where
        Q: Eq + ?Sized + Equivalent<K> + Hash,
        K: Borrow<Q>,
    {
        self.children
            .get_sync(key)
            .map(|occ| occ.get().load(Ordering::Acquire, g))
    }

    pub(crate) fn upsert_child_key<'g, 'i, Q>(&'g self, g: &'g Guard, item: &'i Q) -> Ptr<'g,Node<K,V>>
    where
        K: Borrow<Q>,
        Q: Eq + Equivalent<K> + Hash + ToOwned<Owned=K> + ?Sized + 'i,
    {
        let key: K = item.to_owned();
        let occupied = self.children.entry_sync(key)
            .or_insert_with(|| AtomicShared::new(Node::default()));
        ensure_not_null(occupied.get(), g)
    }

    pub(crate) fn set_here_value<'g>(&self, g: &'g Guard, value: V) -> Option<Shared<V>> {
        let mut new = Shared::new(value);
        let mut old = self.here.load(Ordering::Relaxed, &g);
        loop {
            let guard = Guard::new();
            match self.here.compare_exchange_weak(
                old,
                (Some(new), Tag::None),
                Ordering::AcqRel,
                Ordering::Relaxed,
                &g,
            ) {
                Ok((old,_)) => {
                    return old;
                },
                Err((given,ptr)) => {
                    old = ptr;
                    debug_assert!(given.is_some());
                    new = given.unwrap();
                    // spin lock on this
                    // contention should be extremely low as this is a read
                    // focused data structure
                    std::hint::spin_loop();
                    continue;
                }
            };
        }
    }

    pub(crate) fn remove_here_value<'g>(&self, g: &'g Guard) -> Option<Shared<V>> {
        let mut new: Option<Shared<V>> = None;
        let mut old = self.here.load(Ordering::Relaxed, &g);
        loop {
            let guard = Guard::new();
            match self.here.compare_exchange_weak(
                old,
                (new, Tag::None),
                Ordering::AcqRel,
                Ordering::Relaxed,
                &g,
            ) {
                Ok((old,_)) => {
                    return old;
                },
                Err((given,ptr)) => {
                    old = ptr;
                    debug_assert!(given.is_none());
                    new = None;
                    // spin lock on this
                    // contention should be extremely low as this is a read
                    // focused data structure
                    std::hint::spin_loop();
                    continue;
                }
            };
        }
    }
}
impl<K,V> Node<K,V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync,
{
    pub(crate) fn list_keys<'g>(&self, g: &'g Guard) -> HashSet<K,BuildHasherDefault<SeaHasher>> {
        let mut set = HashSet::default();
        if self.children.is_empty() {
            // avoid atomic overhead of creating an iteration entry
            return set;
        }
        self.children.iter_sync(|key,_| -> bool {
            set.insert(key.clone());
            true
        });
        set
    }

    pub(crate) fn list_keys_recursive<'g>(&self, g: &'g Guard) -> RecursiveListing<K> {
        let mut listing = RecursiveListing::<K>::default();
        if self.children.is_empty() {
            // avoid atomic overhead of creating an iteration entry
            return listing;
        }
        self.children.iter_sync(|key: &K, value: &AtomicShared<Node<K,V>>| -> bool {
            let child_ptr = value.load(Ordering::Acquire, g);
            let child_listing = child_ptr
                .as_ref()
                .map(|p| p.list_keys_recursive(g))
                .unwrap_or_else(RecursiveListing::default);
            listing.children.insert(key.clone(), child_listing);
            true
        });
        listing
    }
}

pub struct RecursiveListing<K> {
    pub children: HashMap<K,Self,BuildHasherDefault<SeaHasher>>,
}
impl<K> Default for RecursiveListing<K>
{
    fn default() -> Self {
        Self {
            children: HashMap::default(),
        }
    }
}


pub(crate) fn ensure_not_null<'g,K,V>(arg: &AtomicShared<Node<K,V>>, guard: &'g Guard) -> Ptr<'g,Node<K,V>>
where
    K: 'static + Send + Sync + Hash + Eq,
    V: 'static + Send + Sync,
{
    let mut current = arg.load(Ordering::Relaxed,guard);
    if !current.is_null() {
        return current;
    }
    let mut shared = Shared::new(Node::default());
    loop {
        match arg.compare_exchange_weak(current, (Some(shared),Tag::None), Ordering::AcqRel, Ordering::Relaxed, guard) {
            Ok((_,state)) => {
                debug_assert!(!state.is_null());
                return state;
            },
            Err((s,new)) => {
                if !new.is_null() {
                    return new;
                }
                current = new;
                shared = s.unwrap_or_else(|| Shared::new(Node::default()));
            }
        };
    }
}
