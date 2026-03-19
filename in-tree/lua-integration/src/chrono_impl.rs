
use chrono::{DateTime, Datelike, Timelike, Utc};
use mlua::{MetaMethod, UserData, UserDataMethods};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChronoWrapper(pub DateTime<Utc>);

impl UserData for ChronoWrapper {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("year", |_, this, ()| Ok(this.0.year()));
        methods.add_method("month", |_, this, ()| Ok(this.0.month()));
        methods.add_method("day", |_, this, ()| Ok(this.0.day()));
        methods.add_method("hour", |_, this, ()| Ok(this.0.hour()));
        methods.add_method("minute", |_, this, ()| Ok(this.0.minute()));
        methods.add_method("second", |_, this, ()| Ok(this.0.second()));
        methods.add_method("seconds_float", |_, this, ()| {
            Ok(this.0.timestamp() as f64 + this.0.timestamp_subsec_nanos() as f64 / 1_000_000_000.0)
        });
        methods.add_method("millis", |_, this, ()| Ok(this.0.timestamp_millis()));
        methods.add_method("millisfloat", |_, this, ()| Ok(this.0.timestamp_millis() as f64));
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(this.0.to_rfc3339())
        });
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for ChronoWrapper {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for ChronoWrapper {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Self(<DateTime<Utc> as serde::de::Deserialize>::deserialize(d)?))
    }
}

impl mlua::IntoLua for ChronoWrapper {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.create_userdata(self)?.into_lua(lua)
    }
}

impl mlua::FromLua for ChronoWrapper {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "ChronoWrapper".to_string(),
                message: None,
            }),
        }
    }
}
