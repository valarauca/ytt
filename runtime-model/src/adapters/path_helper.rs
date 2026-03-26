
use std::{
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

pub trait ServiceRelation<'a> {
    fn creates(&self) -> &'a [&'a str];
    fn requires(&self) -> &'a [&'a [&'a str]];
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


struct SrvInfo<'a> {
    service: &'a dyn ServiceRelation<'a>,
    requires: BTreeSet<usize>,
    dependents: BTreeSet<usize>,
    in_degree: usize,
}
impl<'a> SrvInfo<'a> {
    fn new(service: &'a dyn ServiceRelation<'a>) -> Self {
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
pub fn load_order<'s,I>(services: I) -> anyhow::Result<Vec<&'s dyn ServiceRelation<'s>>>
where
    I: IntoIterator<Item=&'s dyn ServiceRelation<'s>>,
{
    let mut info: Vec<SrvInfo<'s>> = Vec::new();
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
        if reqs.is_empty() {
            continue;
        }
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

