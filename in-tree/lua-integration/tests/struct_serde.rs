#![cfg(feature = "serde")]

use lua_integration::{LuaIntegration, mlua};
use mlua::{Lua, Result};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(LuaIntegration, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[lua(serde)]
pub struct SerdeStruct {
    pub name: String,
    pub value: i32,
}

#[test]
fn test_tostring_serialization() -> Result<()> {
    let lua = Lua::new();

    let data = SerdeStruct {
        name: "test".to_string(),
        value: 42,
    };

    lua.globals().set("data", data)?;

    let json: String = lua.load("return tostring(data)").eval()?;

    let parsed: SerdeStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "test");
    assert_eq!(parsed.value, 42);

    Ok(())
}

#[test]
fn test_from_json_via_rust() -> Result<()> {
    let lua = Lua::new();

    let dummy = SerdeStruct {
        name: "dummy".to_string(),
        value: 0,
    };

    lua.globals().set("obj", dummy)?;

    let json = r#"{"name":"hello","value":123}"#;

    let json_str: String = lua.load(&format!(r#"
        local result_json = obj.from_json('{}')
        return tostring(result_json)
    "#, json)).eval()?;

    let result: SerdeStruct = serde_json::from_str(&json_str).unwrap();
    assert_eq!(result.name, "hello");
    assert_eq!(result.value, 123);

    Ok(())
}

#[test]
fn test_roundtrip() -> Result<()> {
    let lua = Lua::new();

    let original = SerdeStruct {
        name: "roundtrip".to_string(),
        value: 999,
    };

    lua.globals().set("data", original.clone())?;

    let json_str: String = lua.load(r#"
        local json = tostring(data)
        local restored = data.from_json(json)
        return tostring(restored)
    "#).eval()?;

    let result: SerdeStruct = serde_json::from_str(&json_str).unwrap();
    assert_eq!(result, original);

    Ok(())
}
