use lua_integration::{LuaKind, mlua};
use mlua::{Lua, Result};

#[derive(LuaKind, Clone, Debug)]
pub enum SimpleEnum {
    Foo,
    Bar,
    Baz,
}

#[derive(LuaKind, Clone, Debug)]
pub enum RenamedEnum {
    #[lua(rename = "world")]
    World,
    #[lua(rename = "foo")]
    Foo,
    #[lua(rename = "bar")]
    Bar,
}

#[test]
fn test_unit_enum_is_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value", SimpleEnum::Foo)?;

    let is_foo: bool = lua.load("return value:is_foo()").eval()?;
    let is_bar: bool = lua.load("return value:is_bar()").eval()?;
    let is_baz: bool = lua.load("return value:is_baz()").eval()?;

    assert!(is_foo);
    assert!(!is_bar);
    assert!(!is_baz);

    Ok(())
}

#[test]
fn test_unit_enum_renamed_is_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value", RenamedEnum::Foo)?;

    let is_foo: bool = lua.load("return value:is_foo()").eval()?;
    let is_world: bool = lua.load("return value:is_world()").eval()?;

    assert!(is_foo);
    assert!(!is_world);

    Ok(())
}

#[test]
fn test_unit_enum_to_enum_equality() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("value1", SimpleEnum::Foo)?;
    lua.globals().set("value2", SimpleEnum::Foo)?;
    lua.globals().set("value3", SimpleEnum::Bar)?;

    let eq_same: bool = lua.load("return value1 == value2").eval()?;
    let eq_diff: bool = lua.load("return value1 == value3").eval()?;

    assert!(eq_same);
    assert!(!eq_diff);

    Ok(())
}

#[test]
fn test_different_variants_inequality() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("foo", SimpleEnum::Foo)?;
    lua.globals().set("bar", SimpleEnum::Bar)?;
    lua.globals().set("baz", SimpleEnum::Baz)?;

    let foo_ne_bar: bool = lua.load("return foo ~= bar").eval()?;
    let bar_ne_baz: bool = lua.load("return bar ~= baz").eval()?;
    let foo_ne_baz: bool = lua.load("return foo ~= baz").eval()?;

    assert!(foo_ne_bar);
    assert!(bar_ne_baz);
    assert!(foo_ne_baz);

    Ok(())
}
