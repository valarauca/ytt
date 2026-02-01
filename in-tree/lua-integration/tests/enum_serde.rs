#![cfg(feature = "serde")]

use lua_integration::{LuaKind, mlua};
use mlua::{Lua, Result};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(LuaKind, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[lua(serde)]
pub enum SerdeEnum {
    Unit,
    Tuple(i32, String),
    Struct { x: f64, y: f64 },
}

#[test]
fn test_enum_unit_variant_serialize() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value", SerdeEnum::Unit)?;

    let json: String = lua.load("return tostring(value)").eval()?;

    let parsed: SerdeEnum = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, SerdeEnum::Unit);

    Ok(())
}

#[test]
fn test_enum_tuple_variant_serialize() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value", SerdeEnum::Tuple(42, "test".to_string()))?;

    let json: String = lua.load("return tostring(value)").eval()?;

    let parsed: SerdeEnum = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, SerdeEnum::Tuple(42, "test".to_string()));

    Ok(())
}

#[test]
fn test_enum_struct_variant_serialize() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value", SerdeEnum::Struct { x: 3.14, y: 2.71 })?;

    let json: String = lua.load("return tostring(value)").eval()?;

    let parsed: SerdeEnum = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, SerdeEnum::Struct { x: 3.14, y: 2.71 });

    Ok(())
}

#[test]
fn test_enum_from_json() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("dummy", SerdeEnum::Unit)?;

    let json = r#"{"Tuple":[99,"data"]}"#;

    let json_str: String = lua.load(&format!(r#"
        local result = dummy.from_json('{}')
        return tostring(result)
    "#, json)).eval()?;

    let result: SerdeEnum = serde_json::from_str(&json_str).unwrap();
    assert_eq!(result, SerdeEnum::Tuple(99, "data".to_string()));

    Ok(())
}

#[test]
fn test_enum_roundtrip_all_variants() -> Result<()> {
    let lua = Lua::new();

    let test_cases = vec![
        SerdeEnum::Unit,
        SerdeEnum::Tuple(123, "hello".to_string()),
        SerdeEnum::Struct { x: 1.5, y: 2.5 },
    ];

    for original in test_cases {
        lua.globals().set("value", original.clone())?;

        let json_str: String = lua.load(r#"
            local json = tostring(value)
            local restored = value.from_json(json)
            return tostring(restored)
        "#).eval()?;

        let result: SerdeEnum = serde_json::from_str(&json_str).unwrap();
        assert_eq!(result, original);
    }

    Ok(())
}
