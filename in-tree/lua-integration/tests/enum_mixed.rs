use lua_integration::{LuaKind, mlua};
use mlua::{Lua, Result};

#[derive(LuaKind, Clone, Debug)]
pub enum MixedEnum {
    Unit,
    Tuple(i32, String),
    Struct { x: f64, y: f64 },
    #[lua(rename = "single")]
    Single(bool),
}

#[test]
fn test_mixed_enum_all_is_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("unit", MixedEnum::Unit)?;
    lua.globals().set("tuple", MixedEnum::Tuple(42, "test".to_string()))?;
    lua.globals().set("struct_val", MixedEnum::Struct { x: 1.0, y: 2.0 })?;
    lua.globals().set("single", MixedEnum::Single(true))?;

    assert!(lua.load("return unit:is_unit()").eval::<bool>()?);
    assert!(!lua.load("return unit:is_tuple()").eval::<bool>()?);

    assert!(lua.load("return tuple:is_tuple()").eval::<bool>()?);
    assert!(!lua.load("return tuple:is_struct()").eval::<bool>()?);

    assert!(lua.load("return struct_val:is_struct()").eval::<bool>()?);
    assert!(!lua.load("return struct_val:is_single()").eval::<bool>()?);

    assert!(lua.load("return single:is_single()").eval::<bool>()?);
    assert!(!lua.load("return single:is_unit()").eval::<bool>()?);

    Ok(())
}

#[test]
fn test_mixed_enum_tuple_getters() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("tuple", MixedEnum::Tuple(99, "data".to_string()))?;

    let val0: Option<i32> = lua.load("return tuple:get_tuple_0()").eval()?;
    let val1: Option<String> = lua.load("return tuple:get_tuple_1()").eval()?;

    assert_eq!(val0, Some(99));
    assert_eq!(val1, Some("data".to_string()));

    Ok(())
}

#[test]
fn test_mixed_enum_struct_getters() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("struct_val", MixedEnum::Struct { x: 3.14, y: 2.71 })?;

    let x: Option<f64> = lua.load("return struct_val:get_struct_x()").eval()?;
    let y: Option<f64> = lua.load("return struct_val:get_struct_y()").eval()?;

    assert_eq!(x, Some(3.14));
    assert_eq!(y, Some(2.71));

    Ok(())
}

#[test]
fn test_mixed_enum_single_getter() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("single", MixedEnum::Single(false))?;

    let val: Option<bool> = lua.load("return single:get_single()").eval()?;

    assert_eq!(val, Some(false));

    Ok(())
}

#[test]
fn test_mixed_enum_cross_variant_nil() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("unit", MixedEnum::Unit)?;

    let tuple0: Option<i32> = lua.load("return unit:get_tuple_0()").eval()?;
    let struct_x: Option<f64> = lua.load("return unit:get_struct_x()").eval()?;
    let single: Option<bool> = lua.load("return unit:get_single()").eval()?;

    assert_eq!(tuple0, None);
    assert_eq!(struct_x, None);
    assert_eq!(single, None);

    Ok(())
}

#[test]
fn test_mixed_enum_equality() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("unit1", MixedEnum::Unit)?;
    lua.globals().set("unit2", MixedEnum::Unit)?;
    lua.globals().set("tuple", MixedEnum::Tuple(1, "x".to_string()))?;

    let unit_eq: bool = lua.load("return unit1 == unit2").eval()?;
    let unit_ne_tuple: bool = lua.load("return unit1 ~= tuple").eval()?;

    assert!(unit_eq);
    assert!(unit_ne_tuple);

    Ok(())
}
