
use openrouter::{
    completions::{Response as ORResponse, Request as ORRequest},
};

use crate::{
    adapters::{
        service_tree::{RegisteredServiceTree,ServiceFetch},
        s3service::{BoxCloneSyncService},
        service_kind::{ServiceManagement},
    },
};

use mlua::{
    UserData,UserDataMethods,Lua,UserDataRef,IntoLua,FromLua,
};

#[derive(Clone)]
pub struct Tree {
    inner: RegisteredServiceTree,
}
impl UserData for Tree {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {

        async fn get_model_exact(l: Lua, this: UserDataRef<Tree>, key: mlua::String) -> mlua::Result<mlua::Value> {
            let s = key.to_str()?;
            match this.inner.get_service_exact(&*s, ServiceManagement::get_openrouter).await {
                Err(_) => Ok(mlua::Value::Nil),
                Ok(x) => ModelHandle { inner: x }.into_lua(&l),
            }
        }
        m.add_async_method("get_model_exact", get_model_exact);

        async fn get_model(l: Lua, this: UserDataRef<Tree>, key: mlua::String) -> mlua::Result<mlua::Value> {
            let s = key.to_str()?;
            match this.inner.get_service(&*s, ServiceManagement::get_openrouter).await {
                Err(_) => Ok(mlua::Value::Nil),
                Ok(x) => ModelHandle { inner: x }.into_lua(&l),
            }
        }
        m.add_async_method("get_model", get_model_exact);

    }
}


#[derive(Clone)]
pub struct ModelHandle {
    default_prompt: Option<String>,
    inner: BoxCloneSyncService<ORRequest,ORResponse,anyhow::Error>,
}
impl UserData for ModelHandle { }
