
use std::{
    any::{Any},
    path::{Path,PathBuf},
    collections::{BTreeSet,VecDeque},
};

use hashbrown::{HashMap};

pub trait GetTreePath {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error>;
}
impl GetTreePath for str {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self)
    }
}
impl<'b> GetTreePath for &'b str {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self)
    }
}
impl<'b> GetTreePath for &'b &str {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self)
    }
}
impl GetTreePath for String {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self.as_str())
    }
}
impl<'b> GetTreePath for &'b String {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self.as_str())
    }
}
impl<'b> GetTreePath for &'b &String {
    fn get_tree_path<'a>(&'a self) -> Result<Vec<&'a str>,anyhow::Error> {
        split_and_validate_path(self.as_str())
    }
}

fn split_and_validate_path<'a>(arg: &'a str) -> Result<Vec<&'a str>,anyhow::Error> {
    if arg.is_empty() {
        anyhow::bail!("path: '{}' is illegal, must not be an empty string", arg)
    }
    if !arg.starts_with('/') {
        anyhow::bail!("path: '{}' is illegal, must start with '/'", arg)
    }
    if !arg.ends_with('/') {
        anyhow::bail!("path: '{}' is illegal, cannot end with '/'", arg)
    }
    arg.split('/')
        .map(|segment| -> Result<&'a str, anyhow::Error> {
            if segment.is_empty() {
                anyhow::bail!("path: '{}' is illegal, segments may not be empty", arg);
            }
            if segment == "." || segment == ".." {
                anyhow::bail!("path: '{}' is illegal, segements may not contain '.' or '..'", arg);
            }
            Ok(segment)
        })
        .collect()
}

/*
 *
 * Helper methods for determining service load order
 *
 */

trait ServiceRelation<'a> {
    fn creates(&self) -> &'a [&'a str];
    fn requires<'b>(&'b self) -> std::slice::Iter<'b, &'a [&'a str]>;
}

fn path_parents<'a>(path: &'a [&'a str]) -> impl Iterator<Item=&'a [&'a str]> {
    let mut tail = path.len();
    std::iter::from_fn(move || {
        if tail == 0 {
            None
        } else {
            let slice = &path[0..tail];
            tail -= 1;
            Some(slice)
        }
    })
}
#[test]
fn validate_path_parents() {
    let path: &'static [&'static str] = &["foo", "bar", "baz"];
    let mut iter = path_parents(path);
    assert_eq!(iter.next(), Some(["foo", "bar", "baz"].as_ref()));
    assert_eq!(iter.next(), Some(["foo", "bar"].as_ref()));
    assert_eq!(iter.next(), Some(["foo"].as_ref()));
    assert_eq!(iter.next(), None);
}


struct SrvInfo<'a,S> 
where
    S: ServiceRelation<'a> + 'a,
{
    service: &'a S,
    requires: BTreeSet<usize>,
    dependents: BTreeSet<usize>,
    in_degree: usize,
}
impl<'a,S> SrvInfo<'a, S> 
where
    S: ServiceRelation<'a> + 'a,
{
    fn new(service: &'a S) -> Self {
        Self { 
            service,
            in_degree: 0usize,
            requires: BTreeSet::new(),
            dependents: BTreeSet::new(),
        }
    }
}

type SrvReq<'a> = &'a [&'a str];

/// Kahns algorithm for topological sorting
fn load_order<'s,S,I>(services: I) -> anyhow::Result<Vec<&'s S>>
where
    S: ServiceRelation<'s> + 's,
    I: IntoIterator<Item=&'s S>,
{
    let mut info: Vec<SrvInfo<'s, S>> = Vec::new();
    let mut path_to_id: HashMap<&'s [&'s str],usize> = HashMap::new();
    for (num, srv) in services.into_iter().enumerate() {
        debug_assert!(num == info.len() && path_to_id.len() == num);
        let path = srv.creates();
        if let Some(old) = path_to_id.insert(path, num) {
            anyhow::bail!("conflict detected at path: {:#?}, service config indexes: {} & {} provide an identical path", path, num, old);
        }
        info.push(SrvInfo::new(srv));
    }

    let srv_count = info.len();
    for idx in 0..srv_count {
        let reqs = info[idx].service.requires();
        for req in reqs {
            // fuzzy match by searching "up" the tree for prefix matches
            let candidate: usize = path_parents(&req)
                .filter_map(|prefix| path_to_id.get(&prefix).copied())
                .map(|candidate| -> anyhow::Result<usize> {
                    if candidate == idx { 
                        Err(anyhow::anyhow!("service entry {} has dependency {:#?} which is futfilled by itself", idx, &req))
                    } else {
                        Ok(candidate)
                    }
                })
                .next()
                .ok_or_else(|| anyhow::anyhow!("service entry {} has dependency {:#?} which cannot be futfilled", idx, &req))??;

            if info[idx].requires.insert(candidate) {
                // update the dependent conditionally on this being 'new'
                info[candidate].dependents.insert(idx);
            }
        }
        info[idx].in_degree = info[idx].requires.len();
    }

    let mut order = Vec::with_capacity(srv_count);
    let mut queue = VecDeque::with_capacity(srv_count);
    queue.extend(
        info.iter()
            .enumerate()
            .filter(|(_,srv)| srv.in_degree == 0)
            .map(|(idx,_)| idx)
    );

    // allocation to hold dependents
    let mut temp = Vec::with_capacity(srv_count);


    while let Some(idx) = queue.pop_front() {
        if info[idx].in_degree != 0 {
            anyhow::bail!("cyclic dependency amoung services");
        }
        order.push(info[idx].service);


        temp.clear();
        temp.extend(info[idx].dependents.iter().copied());
        for dep in temp.iter().copied() {
            info[dep].in_degree -= 1;
            if info[dep].in_degree == 0 {
                queue.push_back(dep);
            }
        }
    }

    if info.len() != order.len() {
        anyhow::bail!("cyclic dependency amoung services");
    }

    Ok(order)
}




/// Wraps the underlying service configuration
pub struct ServiceConfig(Box<dyn CloneServiceInner>);

impl Clone for ServiceConfig {
    fn clone(&self) -> Self {
        ServiceConfig(self.0.clone_box())
    }
}
impl ServiceConfig {

    pub fn new<T>(arg: T) -> Self
    where
        T: ServiceReqs + Any + Send + Sync + 'static + Clone,
    {
        let x: Box<dyn CloneServiceInner> = Box::new(arg);
        ServiceConfig(x)
    }

    pub fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        self.0.creates()
    }

    pub fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        self.0.requires()
    }

    pub async fn insert_into_tree(&self) -> anyhow::Result<()> {
        self.0.insert_to_tree().await
    }
}

trait CloneServiceInner: ServiceReqs + Send + Sync {
    fn clone_box(&self) -> Box<dyn CloneServiceInner>;
}
impl<T> CloneServiceInner for T
where
    T: ServiceReqs + Clone,
{
    fn clone_box(&self) -> Box<dyn CloneServiceInner> {
        Box::new(self.clone())
    }
}

/// Used to declare what a service needs
pub trait ServiceReqs: Any + Send + Sync + 'static {

    /// Where this service will be inserted into the tree
    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>>;

    /// What this service needs to operate
    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>>;

    /// Inser this type into the tree
    fn insert_to_tree(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=anyhow::Result<()>> + Send + 'static>>;
}

struct Inner<'a> {
    service: &'a ServiceConfig,
    creates: &'a [&'a str],
    requires: Vec<&'a [&'a str]>,
}
impl<'a> ServiceRelation<'a> for Inner<'a> {
    fn creates(&self) -> &'a [&'a str] { self.creates }
    fn requires<'b>(&'b self) -> std::slice::Iter<'b, &'a [&'a str]> {
        self.requires.iter()
    }
}

pub trait IntoServiceConfig {
    fn into_service_config(&self) -> ServiceConfig;
}

/// Loads all services in the correct order
pub async fn loader<'a>(services: &'a [ServiceConfig]) -> anyhow::Result<()> {

    let mut repr = Vec::<(&'a ServiceConfig, Vec<&'a str>, Vec<Vec<&'a str>>)>::with_capacity(services.len());
    for s in services.iter() {
        let creates = s.creates()?;
        let requires = s.requires()?;
        repr.push((s, creates, requires));
    }

    // needs to borrow from repr so we loop again
    let mut ordering = Vec::with_capacity(services.len());
    for idx in 0..services.len() {
        ordering.push(Inner {
            service: repr[idx].0,
            creates: &repr[idx].1,
            requires: repr[idx].2.as_slice().iter().map(|x| x.as_slice()).collect(),
        });
    }

    // topological sort borrows the borrowed data
    // avoids a lot of branches 
    for item in load_order(ordering.iter())? {
        // items are loaded in topological order
        // sequentially as we've lost dependency information
        item.service.insert_into_tree().await?;
    }

    Ok(())
}
