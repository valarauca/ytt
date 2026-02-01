use lua_integration::{LuaKind, mlua};
use mlua::{Lua, Result};

#[derive(LuaKind, Clone, Debug)]
pub enum StructVariant {
    #[lua(rename = "point")]
    Point { x: f64, y: f64 },
    #[lua(rename = "person")]
    Person { name: String, age: i32 },
    #[lua(rename = "config")]
    Config { enabled: bool, threshold: f64, label: String },
}

#[test]
fn test_struct_variant_is_variant() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("point", StructVariant::Point { x: 1.0, y: 2.0 })?;
    lua.globals().set("person", StructVariant::Person {
        name: "Alice".to_string(),
        age: 30
    })?;

    let point_is_point: bool = lua.load("return point:is_point()").eval()?;
    let point_is_person: bool = lua.load("return point:is_person()").eval()?;
    let person_is_person: bool = lua.load("return person:is_person()").eval()?;
    let person_is_point: bool = lua.load("return person:is_point()").eval()?;

    assert!(point_is_point);
    assert!(!point_is_person);
    assert!(person_is_person);
    assert!(!person_is_point);

    Ok(())
}

#[test]
fn test_struct_variant_get_fields() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("point", StructVariant::Point { x: 3.14, y: 2.71 })?;

    let x: Option<f64> = lua.load("return point:get_point_x()").eval()?;
    let y: Option<f64> = lua.load("return point:get_point_y()").eval()?;

    assert_eq!(x, Some(3.14));
    assert_eq!(y, Some(2.71));

    Ok(())
}

#[test]
fn test_struct_variant_get_wrong_variant_fields() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("point", StructVariant::Point { x: 1.0, y: 2.0 })?;

    let name: Option<String> = lua.load("return point:get_person_name()").eval()?;
    let age: Option<i32> = lua.load("return point:get_person_age()").eval()?;

    assert_eq!(name, None);
    assert_eq!(age, None);

    Ok(())
}

#[test]
fn test_struct_variant_multiple_fields() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("config", StructVariant::Config {
        enabled: true,
        threshold: 0.95,
        label: "production".to_string(),
    })?;

    let enabled: Option<bool> = lua.load("return config:get_config_enabled()").eval()?;
    let threshold: Option<f64> = lua.load("return config:get_config_threshold()").eval()?;
    let label: Option<String> = lua.load("return config:get_config_label()").eval()?;

    assert_eq!(enabled, Some(true));
    assert_eq!(threshold, Some(0.95));
    assert_eq!(label, Some("production".to_string()));

    Ok(())
}

#[test]
fn test_struct_variant_person_fields() -> Result<()> {
    let lua = Lua::new();

    lua.globals().set("person", StructVariant::Person {
        name: "Bob".to_string(),
        age: 25,
    })?;

    let name: Option<String> = lua.load("return person:get_person_name()").eval()?;
    let age: Option<i32> = lua.load("return person:get_person_age()").eval()?;

    assert_eq!(name, Some("Bob".to_string()));
    assert_eq!(age, Some(25));

    let x: Option<f64> = lua.load("return person:get_point_x()").eval()?;
    assert_eq!(x, None);

    Ok(())
}
