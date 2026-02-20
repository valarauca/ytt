use lua_integration::{LuaIntegration, mlua};
use mlua::{Lua, Result};

#[derive(LuaIntegration, Clone)]
pub struct BasicStruct {
    pub foo: bool,
    pub bar: usize,
    baz: String,
}

#[test]
fn test_public_field_read() -> Result<()> {
    let lua = Lua::new();

    let data = BasicStruct {
        foo: true,
        bar: 42,
        baz: "private".to_string(),
    };

    lua.globals().set("data", data)?;

    let foo: bool = lua.load("return data.foo").eval()?;
    assert_eq!(foo, true);

    let bar: usize = lua.load("return data.bar").eval()?;
    assert_eq!(bar, 42);

    Ok(())
}

#[test]
fn test_public_field_write() -> Result<()> {
    let lua = Lua::new();

    let data = BasicStruct {
        foo: false,
        bar: 0,
        baz: "private".to_string(),
    };

    lua.globals().set("data", data)?;

    lua.load("data.foo = true").exec()?;
    lua.load("data.bar = 99").exec()?;

    let foo: bool = lua.load("return data.foo").eval()?;
    assert_eq!(foo, true);

    let bar: usize = lua.load("return data.bar").eval()?;
    assert_eq!(bar, 99);

    Ok(())
}

#[test]
fn test_private_field_not_accessible() -> Result<()> {
    let lua = Lua::new();

    let data = BasicStruct {
        foo: true,
        bar: 42,
        baz: "private".to_string(),
    };

    lua.globals().set("data", data)?;

    let result = lua.load("return data.baz").eval::<mlua::Value>();
    assert!(result.is_err() || matches!(result.unwrap(), mlua::Value::Nil));

    Ok(())
}

#[derive(LuaIntegration, Clone)]
pub struct Foo {
    pub bar: Bar,
}
#[derive(LuaIntegration, Clone)]
pub struct Bar {
    pub a: usize,
    pub b: usize,
}

#[test]
fn test_nested_lua_integration_read() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("foo", Foo {
        bar: Bar { a: 1, b: 2 },
    })?;

    let a: usize = lua.load("return foo.bar.a").eval()?;
    assert_eq!(a, 1);

    let b: usize = lua.load("return foo.bar.b").eval()?;
    assert_eq!(b, 2);

    Ok(())
}

#[test]
fn test_nested_lua_integration_write() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("foo", Foo {
        bar: Bar { a: 0, b: 0 },
    })?;
    lua.globals().set("new_bar", Bar { a: 10, b: 20 })?;

    lua.load("foo.bar = new_bar").exec()?;

    let a: usize = lua.load("return foo.bar.a").eval()?;
    assert_eq!(a, 10);

    let b: usize = lua.load("return foo.bar.b").eval()?;
    assert_eq!(b, 20);

    Ok(())
}
