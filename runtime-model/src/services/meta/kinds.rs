use std::{
    sync::Arc,
    collections::{HashSet},
    hash::{BuildHasherDefault},
    marker::PhantomData,
};
use seahash::SeaHasher;
use tokio::sync::RwLock;

use crate::traits::{RegisteredService,Err,BoxedConfig,ServiceObj};
use tree::{RecursiveListing};






/*
pub struct ServiceGetRequest<E: Err>{
    pub path: Vec<String>,
    _marker: PhantomData<fn(E)>,
}
pub struct ServiceGetResponse<E: Err> {
    pub arg: Option<Arc<RwLock<Box<dyn RegisteredService<E>>>>>,
    _marker: PhantomData<fn(E)>,
}

pub struct ServiceGetOrParentRequest<E: Err> {
    pub path: Vec<String>,
    _marker: PhantomData<fn(E)>,
}
pub struct ServiceGetOrParentResponse<E: Err> {
    pub arg: Option<Arc<RwLock<Box<dyn RegisteredService<E>>>>>,
    _marker: PhantomData<fn(E)>,
}

pub struct ServiceRemoveRequest {
    pub path: Vec<String>,
}
pub struct ServiceRemoveResponse {
    pub something_was_removed: bool,
}

pub struct ServiceListChildrenRequest {
    pub path: Vec<String>,
}
pub struct ServiceListChildrenResponse {
    pub info: Option<HashSet<String, BuildHasherDefault<SeaHasher>>>
}

pub struct ServiceListChildrenRecursiveRequest {
    pub path: Vec<String>,
}
pub struct ServiceListChildrenRecursiveResponse {
    pub info: Option<RecursiveListing<String>>,
}
*/
