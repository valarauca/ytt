use lua_integration::{LuaIntegration, mlua};
use mlua::{Lua, Result};

#[derive(LuaIntegration, Clone, PartialEq, Debug)]
#[lua(Eq)]
pub struct EqStruct {
    pub value: i32,
    pub name: String,
}

#[test]
fn test_eq_same_values() -> Result<()> {
    let lua = Lua::new();

    let data1 = EqStruct {
        value: 42,
        name: "test".to_string(),
    };

    let data2 = EqStruct {
        value: 42,
        name: "test".to_string(),
    };

    lua.globals().set("data1", data1)?;
    lua.globals().set("data2", data2)?;

    let result: bool = lua.load("return data1 == data2").eval()?;
    assert!(result);

    Ok(())
}

#[test]
fn test_eq_different_values() -> Result<()> {
    let lua = Lua::new();

    let data1 = EqStruct {
        value: 42,
        name: "test".to_string(),
    };

    let data2 = EqStruct {
        value: 99,
        name: "different".to_string(),
    };

    lua.globals().set("data1", data1)?;
    lua.globals().set("data2", data2)?;

    let result: bool = lua.load("return data1 == data2").eval()?;
    assert!(!result);

    Ok(())
}

#[test]
fn test_eq_partial_difference() -> Result<()> {
    let lua = Lua::new();

    let data1 = EqStruct {
        value: 42,
        name: "test".to_string(),
    };

    let data2 = EqStruct {
        value: 42,
        name: "different".to_string(),
    };

    lua.globals().set("data1", data1)?;
    lua.globals().set("data2", data2)?;

    let result: bool = lua.load("return data1 == data2").eval()?;
    assert!(!result);

    Ok(())
}

#[test]
fn test_neq_operator() -> Result<()> {
    let lua = Lua::new();

    let data1 = EqStruct {
        value: 1,
        name: "a".to_string(),
    };

    let data2 = EqStruct {
        value: 2,
        name: "b".to_string(),
    };

    lua.globals().set("data1", data1)?;
    lua.globals().set("data2", data2)?;

    let result: bool = lua.load("return data1 ~= data2").eval()?;
    assert!(result);

    Ok(())
}
