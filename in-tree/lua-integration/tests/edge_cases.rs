use lua_integration::{LuaIntegration, LuaKind, mlua};
use mlua::{Lua, Result};

#[derive(LuaIntegration, Clone)]
pub struct EmptyStruct {}

#[derive(LuaIntegration, Clone)]
pub struct NoPublicFields {
    private: i32,
}

#[derive(LuaKind, Clone)]
pub enum SingleVariant {
    Only,
}

#[derive(LuaKind, Clone)]
pub enum SingleTupleVariant {
    Value(String),
}

#[test]
fn test_empty_struct() -> Result<()> {
    let lua = Lua::new();
    lua.globals().set("empty", EmptyStruct {})?;
    assert!(lua.load("return empty").eval::<mlua::Value>().is_ok());
    Ok(())
}

#[test]
fn test_no_public_fields() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("data", NoPublicFields { private: 42 })?;

    let result = lua.load("return data.private").eval::<mlua::Value>();
    assert!(result.is_err() || matches!(result.unwrap(), mlua::Value::Nil));

    Ok(())
}

#[test]
fn test_single_variant_enum() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value", SingleVariant::Only)?;

    let is_only: bool = lua.load("return value:is_only()").eval()?;
    assert!(is_only);

    Ok(())
}

#[test]
fn test_single_variant_equality() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("v1", SingleVariant::Only)?;
    lua.globals().set("v2", SingleVariant::Only)?;

    let eq: bool = lua.load("return v1 == v2").eval()?;
    assert!(eq);

    Ok(())
}

#[test]
fn test_single_tuple_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value", SingleTupleVariant::Value("test".to_string()))?;

    let is_value: bool = lua.load("return value:is_value()").eval()?;
    assert!(is_value);

    let val: Option<String> = lua.load("return value:get_value()").eval()?;
    assert_eq!(val, Some("test".to_string()));

    Ok(())
}

#[derive(LuaIntegration, Clone)]
pub struct NestedTypes {
    pub vec_field: Vec<i32>,
    pub option_field: Option<String>,
}

#[test]
fn test_nested_types() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("data", NestedTypes {
        vec_field: vec![1, 2, 3],
        option_field: Some("hello".to_string()),
    })?;

    let vec: Vec<i32> = lua.load("return data.vec_field").eval()?;
    assert_eq!(vec, vec![1, 2, 3]);

    let opt: Option<String> = lua.load("return data.option_field").eval()?;
    assert_eq!(opt, Some("hello".to_string()));

    Ok(())
}

#[test]
fn test_option_none() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("data", NestedTypes {
        vec_field: vec![],
        option_field: None,
    })?;

    let opt: Option<String> = lua.load("return data.option_field").eval()?;
    assert_eq!(opt, None);

    Ok(())
}

#[derive(LuaKind, Clone)]
pub enum ComplexEnum {
    A,
    B(i32),
    C { x: i32 },
    D(i32, i32),
    E { a: i32, b: i32 },
}

#[test]
fn test_complex_enum_all_variants() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("a", ComplexEnum::A)?;
    lua.globals().set("b", ComplexEnum::B(10))?;
    lua.globals().set("c", ComplexEnum::C { x: 20 })?;
    lua.globals().set("d", ComplexEnum::D(30, 40))?;
    lua.globals().set("e", ComplexEnum::E { a: 50, b: 60 })?;

    assert!(lua.load("return a:is_a()").eval::<bool>()?);
    assert!(lua.load("return b:is_b()").eval::<bool>()?);
    assert!(lua.load("return c:is_c()").eval::<bool>()?);
    assert!(lua.load("return d:is_d()").eval::<bool>()?);
    assert!(lua.load("return e:is_e()").eval::<bool>()?);

    assert_eq!(lua.load("return b:get_b()").eval::<Option<i32>>()?, Some(10));
    assert_eq!(lua.load("return c:get_c_x()").eval::<Option<i32>>()?, Some(20));
    assert_eq!(lua.load("return d:get_d_0()").eval::<Option<i32>>()?, Some(30));
    assert_eq!(lua.load("return d:get_d_1()").eval::<Option<i32>>()?, Some(40));
    assert_eq!(lua.load("return e:get_e_a()").eval::<Option<i32>>()?, Some(50));
    assert_eq!(lua.load("return e:get_e_b()").eval::<Option<i32>>()?, Some(60));

    Ok(())
}
