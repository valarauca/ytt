use lua_integration::{LuaKind, mlua};
use mlua::{Lua, Result};

#[derive(LuaKind, Clone, Debug)]
pub enum TupleSingle {
    #[lua(rename = "value")]
    Value(i32),
    #[lua(rename = "text")]
    Text(String),
}

#[derive(LuaKind, Clone, Debug)]
pub enum TupleMulti {
    Pair(i32, String),
    Triple(bool, f64, i32),
}

#[test]
fn test_tuple_single_is_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("val", TupleSingle::Value(42))?;
    lua.globals().set("txt", TupleSingle::Text("hello".to_string()))?;

    let val_is_value: bool = lua.load("return val:is_value()").eval()?;
    let val_is_text: bool = lua.load("return val:is_text()").eval()?;
    let txt_is_value: bool = lua.load("return txt:is_value()").eval()?;
    let txt_is_text: bool = lua.load("return txt:is_text()").eval()?;

    assert!(val_is_value);
    assert!(!val_is_text);
    assert!(!txt_is_value);
    assert!(txt_is_text);

    Ok(())
}

#[test]
fn test_tuple_single_get_value() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("val", TupleSingle::Value(42))?;

    let value: Option<i32> = lua.load("return val:get_value()").eval()?;
    assert_eq!(value, Some(42));

    let text: Option<String> = lua.load("return val:get_text()").eval()?;
    assert_eq!(text, None);

    Ok(())
}

#[test]
fn test_tuple_single_get_wrong_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("txt", TupleSingle::Text("hello".to_string()))?;

    let value: Option<i32> = lua.load("return txt:get_value()").eval()?;
    assert_eq!(value, None);

    let text: Option<String> = lua.load("return txt:get_text()").eval()?;
    assert_eq!(text, Some("hello".to_string()));

    Ok(())
}

#[test]
fn test_tuple_multi_is_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("pair", TupleMulti::Pair(10, "test".to_string()))?;

    let is_pair: bool = lua.load("return pair:is_pair()").eval()?;
    let is_triple: bool = lua.load("return pair:is_triple()").eval()?;

    assert!(is_pair);
    assert!(!is_triple);

    Ok(())
}

#[test]
fn test_tuple_multi_get_indexed() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("pair", TupleMulti::Pair(99, "data".to_string()))?;

    let val0: Option<i32> = lua.load("return pair:get_pair_0()").eval()?;
    let val1: Option<String> = lua.load("return pair:get_pair_1()").eval()?;

    assert_eq!(val0, Some(99));
    assert_eq!(val1, Some("data".to_string()));

    Ok(())
}

#[test]
fn test_tuple_triple_get_indexed() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("triple", TupleMulti::Triple(true, 3.14, 100))?;

    let val0: Option<bool> = lua.load("return triple:get_triple_0()").eval()?;
    let val1: Option<f64> = lua.load("return triple:get_triple_1()").eval()?;
    let val2: Option<i32> = lua.load("return triple:get_triple_2()").eval()?;

    assert_eq!(val0, Some(true));
    assert_eq!(val1, Some(3.14));
    assert_eq!(val2, Some(100));

    Ok(())
}

#[test]
fn test_tuple_wrong_variant_returns_nil() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("pair", TupleMulti::Pair(1, "x".to_string()))?;

    let val0: Option<bool> = lua.load("return pair:get_triple_0()").eval()?;
    let val1: Option<f64> = lua.load("return pair:get_triple_1()").eval()?;
    let val2: Option<i32> = lua.load("return pair:get_triple_2()").eval()?;

    assert_eq!(val0, None);
    assert_eq!(val1, None);
    assert_eq!(val2, None);

    Ok(())
}
