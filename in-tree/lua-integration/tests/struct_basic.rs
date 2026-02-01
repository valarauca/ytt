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
