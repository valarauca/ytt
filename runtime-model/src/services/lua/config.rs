use std::collections::HashMap;

use serde::{Deserialize,Serialize};
use serde_json::{Value as JSValue};
use anyhow::Context;
use mlua::{LuaSerdeExt};
use crate::adapters::{
    path_helper::{GetTreePath,ServiceReqs,IntoServiceConfig,ServiceConfig},
    service_tree::{get_tree},
};

use config_crap::{
    env::{WithEnv},
    boolean::{Boolean},
};

use super::repr::load_client;

#[derive(Clone,Serialize,Deserialize,PartialEq,Debug)]
pub struct BasicLuaRuntimeConfig {
    pub path: String,
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<Vec<String>>,
    #[serde(default)]
    pub lua_options: LuaConfigOption,
}
impl IntoServiceConfig for BasicLuaRuntimeConfig {
    fn into_service_config(&self) -> ServiceConfig {
        ServiceConfig::new(self.clone())
    }
}
impl ServiceReqs for BasicLuaRuntimeConfig {

    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        self.path.get_tree_path()
    }

    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        Ok(Vec::new())
    }

    fn insert_to_tree(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=anyhow::Result<()>> + Send + 'static>> {
        let s = self.clone();
        Box::pin(async move {
            let path = s.path.clone();
            let tree = get_tree();
            if tree.contains_path(&path)? {
                tree.reload(&path, Box::new(s)).await?;
            } else {
                load_client(tree, s)?;
            }
            Ok(())
        })
    }
}

#[derive(Clone,Serialize,Deserialize,PartialEq,Debug,Default)]
pub struct LuaConfigOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<Boolean>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_env: Option<HashMap<String,JSValue>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_os: Option<Boolean>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<HashMap<String,JSValue>>,
}

fn get_bool(opt: &Option<Boolean>) -> bool {
    opt.as_ref().map(|b| b.as_bool()).unwrap_or(false)
}

impl LuaConfigOption {

    pub fn initialize(&self, name: &str, code: &str) -> anyhow::Result<(mlua::Lua,HashMap<String,mlua::Function>)> {
        let mut stdlib = mlua::StdLib::ALL_SAFE;
        if get_bool(&self.disable_os) {
            stdlib ^= mlua::StdLib::OS;
        }
        let lua = mlua::Lua::new_with(stdlib, mlua::LuaOptions::default())
            .context("failed to initialize lua runtime")?;
        if let Some(limit) = &self.memory_limit {
            lua.set_memory_limit(*limit)
                .with_context(|| format!("failed to set memory limit: '{}'", *limit))?;
        }
        if let Some(env) = &self.default_env {
            let globals = lua.globals();
            for (k,v) in env {
                let k = k.as_str();
                let v = lua.to_value(v).with_context(|| format!("failed to set env key '{}'", k))?;
                globals.set(k,v).with_context(|| format!("failed to set env key '{}'", k))?;
            }
        }
        if get_bool(&self.sandbox) {
            lua.sandbox(true)?;
        }

        let compiler = mlua::Compiler::new()
            .set_optimization_level(2)
            .set_debug_level(1);
        let chunk_env: mlua::Table = lua.create_table()?;
        let mt: mlua::Table = lua.create_table()?;
        mt.set("__index", lua.globals())
            .context("failed to set __index")?;
        chunk_env.set_metatable(Some(mt))
            .context("failed to set metatable")?;

        lua.load(code)
            .set_name(format!("@{}", name))
            .set_compiler(compiler)
            .set_environment(chunk_env.clone())
            .exec()
            .context("failed to run user code to attain symbols")?;


        let mut functions = HashMap::new();
        // error doesn't matter
        let _ = chunk_env.for_each(|k: mlua::Value, v: mlua::Value| -> mlua::Result<()> {
            let (s,f) = match (k,v) {
                (mlua::Value::String(s),mlua::Value::Function(f)) => {
                    (s,f)
                },
                _ => return Ok(())
            };
            functions.insert(s.to_string_lossy(), f);
            Ok(())
        });

        Ok((lua, functions))
    }

	/// used for testing
	pub(crate) fn inject_env(&self, k: &str, v: JSValue) -> Self {
		let mut s = self.clone();
		if s.default_env.is_none() {
			s.default_env = Some(HashMap::new());
		}
		if let Some(env) = &mut s.default_env {
			env.insert(k.to_string(), v);
		}
		s
	}
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_json::{Value as JSValue};

    #[test]
    fn test_env_gate_features() {
        const CODE: &'static str = r#"
			if some_variable then
			    function foo()
			        return "hello from foo"
			    end
			else
			    function bar()
			        return "hello from bar"
			    end
			end "#;
        let base_config = LuaConfigOption::default();

		let true_config = base_config.inject_env("some_variable", JSValue::from(true));
		let (_,funcs) = true_config.initialize("idk", CODE).unwrap();
		assert!(funcs.contains_key("foo"));

		let false_config = base_config.inject_env("some_variable", JSValue::from(false));
		let (_,funcs) = false_config.initialize("idk", CODE).unwrap();
		assert!(funcs.contains_key("bar"));
    }
}

