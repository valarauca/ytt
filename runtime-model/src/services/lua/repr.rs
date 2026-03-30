
use std::{
    task::{Context,Poll},
    sync::{Arc},
    ops::Deref,
    collections::HashMap,
};
use tower::{Service,service_fn};
use mlua::{Function,serde::{LuaSerdeExt}};
use serde_json::{Value as JSValue};
use scc::HashIndex;

use crate::{
    adapters::reconfigurable::ReconfigurableService,
    adapters::service_tree::RegisteredServiceTree,
    adapters::maybe_async::{MaybeFuture,make_boxed,make_ready},
    adapters::service_kind::ServiceManagement,
};
use super::config::{BasicLuaRuntimeConfig, LuaConfigOption};


pub fn load_client(
    tree: RegisteredServiceTree,
    config: BasicLuaRuntimeConfig,
) -> anyhow::Result<()> {
    let path = config.path.clone();
    let r = ReconfigurableService::new(config, 1, service_fn(factory_impl));
    let manager = ServiceManagement::from(r);
    tree.insert(&path, manager)?;
    Ok(())
}

async fn factory_impl(config: BasicLuaRuntimeConfig) -> anyhow::Result<LuaRT> {
    let (runtime, map) = config.lua_options.initialize(&config.path,&config.code, &config.public)?;
    let info = scc::HashIndex::new();
    for (k,v) in map.into_iter() {
        let valid = config.lua_options.validation.as_ref().map(|x: &HashMap<String,JSValue>| x.get(&k)).flatten();
        let _ = info.insert_sync(k, LuaCallInfo::new(v,valid));
    }
    Ok(LuaRT {
        lua: runtime,
        funcs: Arc::new(info),
    })
}


#[derive(Clone)]
pub struct LuaRT {
    lua: mlua::Lua,
    funcs: Arc<scc::HashIndex<String,LuaCallInfo>>,
}

#[derive(Clone)]
struct LuaCallInfo {
    func: mlua::Function,
    validate: Option<Arc<JSValue>>,
    is_var_args: bool,
    args: usize,
}
impl LuaCallInfo {
    fn new(func: mlua::Function, validate: Option<&JSValue>) -> Self {
        let info = func.info();
        Self {
            func: func,
            is_var_args: info.is_vararg,
            args: info.num_params as usize,
            validate: validate.map(|j| Arc::new(j.clone())),
        }
    }
}

/// A request to the lua runtime
pub struct LuaCall {
    pub name: String,
    pub args: JSValue,
    pub env: HashMap<String,JSValue>,
}

impl LuaRT {
    async fn call(&self, req: LuaCall) -> anyhow::Result<JSValue> {
        use anyhow::Context;

        if !req.args.is_null() || req.args.is_array() {
            anyhow::bail!("invalid call, arguments must be an JSON array or null");
        }

        let info = self.funcs.deref().get_sync(&req.name)
            .ok_or_else(|| anyhow::anyhow!("could not find function: '{}'", &req.name))?
            .get()
            .clone();
        // validate number of arguments
        match (info.is_var_args, info.args, &req.args) {
            (true,_,_) => { },
            (false, 0, JSValue::Null) => { },
            (false, x, JSValue::Null) => {
                anyhow::bail!("invalid call, caller passed `null` args, function expects '{}'", x);
            },
            (false, x, JSValue::Array(vec)) => {
                if x != vec.len() {
                    anyhow::bail!("invalid call, caller has '{}' args, function expects '{}'", vec.len(), x);
                }
            },
            _ => {
                anyhow::bail!("invalid call, arguments must be an JSON array or null");
            }
        };

        let (f,v) = (info.func,info.validate);


        // do json schema validation if it is so configured
        if let Some(validate) = v.as_ref() {
            let eval = jsonschema::evaluate(&validate, &req.args);
            if !eval.flag().valid {
                return Err(eval.iter_errors().fold(anyhow::anyhow!("json validation failed for function '{}'", &req.name), |msg, err| {
                    msg.context(format!("{:?}", err))
                }));
            }
        }

        let func = if !req.env.is_empty() {
            let t = self.lua.create_table()?;
            for (k,v) in req.env {
                let k = k.as_str();
                let v = self.lua.to_value(&v).with_context(|| format!("failed to set env key '{}'", k))?;
                t.set(k,v).with_context(|| format!("failed to set env key '{}'", k))?;
            }
            // deep clone to ensure that our environment changes are isolated
            let func = f.deep_clone()
                .with_context(|| format!("failed to deep clone the function '{}'", &req.name))?;
            func.set_environment(t)?;
            func
        } else {
            f
        };


        let args = match req.args {
            JSValue::Null => vec![mlua::Value::Nil],
            JSValue::Array(args_vec) => {
                let mut args: Vec<mlua::Value> = Vec::with_capacity(args_vec.len());
                for (idx,arg) in args_vec.into_iter().enumerate() {
                    let v = self.lua.to_value(&arg)
                        .with_context(|| format!("failed to convert argument '{}' to lua", idx))?;
                    args.push(v);
                }
                args
            },
            _ => {
                // this is unreachable but w.e
                anyhow::bail!("invalid call, arguments must be an JSON array or null");
            }
        };
        let args = mlua::MultiValue::from_vec(args);
        let out: mlua::Value = func.call_async(args).await?;
        Ok(self.lua.from_value::<JSValue>(out)?)
    }
}
impl Service<LuaCall> for LuaRT {
    type Response = JSValue;
    type Error = anyhow::Error;
    type Future = MaybeFuture<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: LuaCall) -> Self::Future {
        let this = self.clone();
        make_boxed(async move {
            this.call(req).await
        })
    }
}

