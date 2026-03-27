use std::{
    ops::Deref,
    pin::Pin,
    future::Future,
};

use anyhow::{Error};
use tower::{Service,ServiceExt};
use mlua::{UserData,UserDataMethods,Lua,UserDataRef};
use futures_util::{FutureExt,TryFutureExt};

use lua_integration::{LuaKind,LuaIntegration};
use openrouter::completions::{Request as ORRequest,Response as ORResponse};
use runtime_model::adapters::{BoxCloneSyncService};


#[derive(Clone)]
pub struct ORService {
    internals: BoxCloneSyncService<ORRequest,ORResponse,anyhow::Error>,
}
impl From<BoxCloneSyncService<ORRequest,ORResponse,anyhow::Error>> for ORService {
    fn from(internals: BoxCloneSyncService<ORRequest,ORResponse,anyhow::Error>) -> Self {
        Self { internals }
    }
}
impl UserData for ORService {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_async_method("make_full_request", |_: mlua::Lua, this: UserDataRef<Self>, req: UserDataRef<ORRequest>| -> Pin<Box<dyn Future<Output=mlua::Result<ORResponse>> + Send + 'static>> {
            let service = this.deref().internals.clone(); 
            let request: ORRequest = req.deref().clone();
            invoke_service(service, request, mlua::Error::external)
        });
    }
}


pub type PBFut<T> = Pin<Box<dyn Future<Output=T> + Send + 'static>>;
pub fn invoke_service<S,R,E>(mut service: S, request: R, change: fn(<S as Service<R>>::Error) -> E) -> PBFut<Result<<S as Service<R>>::Response,E>>
where
    S: Service<R> + ServiceExt<R> + Send + 'static,
    <S as Service<R>>::Future: Send + 'static,
    <S as Service<R>>::Response: Send + 'static,
    <S as Service<R>>::Error: Send + 'static,
    E: Send + 'static,
    R: Send + 'static,
{
    let err_lambda = move |x: Result<<S as Service<R>>::Response,<S as Service<R>>::Error>| -> Result<<S as Service<R>>::Response,E> {
        x.map_err(change)
    };
    Box::pin(async move {
        service.ready().then(move |r: Result<&mut S,<S as Service<R>>::Error>| {
            match r {
                Ok(srv) => {
                    srv.call(request).map(err_lambda).left_future()
                }
                Err(e) => {
                    std::future::ready(Err(change(e))).right_future()
                }
            }
        }).await
    })
}

