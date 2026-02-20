use std::{
    ops::Deref,
    pin::Pin,
    future::Future,
};

use anyhow::{Error};
use tower::{Service,ServiceExt};
use mlua::{UserData,UserDataMethods,Lua,UserDataRef};
use futures_util::{FutureExt,TryFutureExt};

use lua_integration::{LuaKind};
use openrouter::completions::{Request as ORRequest,Response as ORResponse};
use runtime_model::adapters::{BoxCloneSyncService};


#[derive(Clone)]
pub struct ORService {
    internals: BoxCloneSyncService<ORRequest,ORResponse,Error>,
}
impl From<BoxCloneSyncService<ORRequest,ORResponse,Error>> for ORService {
    fn from(internals: BoxCloneSyncService<ORRequest,ORResponse,Error>) -> Self {
        Self { internals }
    }
}

#[derive(Clone,PartialEq,LuaKind)]
pub enum Response {
    Success(ORResponse),
    Error(String),
}

impl UserData for ORService {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_async_method("call", |_: &L, this: UserDataRef<Self>, A: UserDataRef<ORRequest>| -> Pin<Box<dyn Future<Output=mlua::Result<Response>> + Send + 'static>> {
            let mut this: Self = this.deref().clone();
            Box::pin(async move {
                let mut service: BoxCloneSyncService<ORRequest,ORResponse,Error> = this.internals;
                let out = match service.ready().and_then(move |x| x.call(a.deref().clone())).await {
                    Ok(x) => Response::Success(ORResponse),
                    Err(e) => {
                        // TODO: log this
                        Response::Error(format!("{:?}", e))
                    }
                };
                Ok(out)
            })
        })
    }
}
