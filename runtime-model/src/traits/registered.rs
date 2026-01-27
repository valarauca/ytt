use std::{
    pin::Pin,
    future::Future,
    any::Any,
};
use tower::{Service,MakeService};

use reloadable::{ReloadingInstance,ReloadableService};
use super::errors::{Err};
use crate::{
    adapters::maybe_async::{MaybeFuture,make_ready},
    services::web_request::service_impl::{UncachedClient,ClientCloner},
};

