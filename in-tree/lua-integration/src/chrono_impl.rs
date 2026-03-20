
use chrono::{DateTime, Datelike, Timelike, Utc};
use mlua::{MetaMethod, UserData, UserDataMethods,UserDataRef};
#[cfg(feature="serde")] use serde::{Serialize,Deserialize};

use crate::traits::{LuaSetterArg};


#[cfg_attr(feature = "serde", derive(Clone, Debug, PartialEq, Eq, Hash,Serialize,Deserialize))]
#[cfg_attr(not(feature = "serde"), derive(Clone, Debug, PartialEq, Eq, Hash))]
#[cfg_attr(feature="serde", serde(transparent))]
pub struct ChronoWrapper {
    #[cfg_attr(feature = "serde", serde(with="chrono::serde::ts_seconds"))]
    pub inner: DateTime<Utc>
}
impl ChronoWrapper {
    pub fn new_utc(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<Self> {
        use chrono::{NaiveDate,NaiveTime,NaiveDateTime};
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_opt(hour, min, sec)?;
        Some(Self {
            inner: DateTime::from_naive_utc_and_offset(NaiveDateTime::new(date,time), Utc),
        })
    }
    pub fn new_utc_micros(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32, micro: u32) -> Option<Self> {
        use chrono::{NaiveDate,NaiveTime,NaiveDateTime};
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_micro_opt(hour, min, sec, micro)?;
        Some(Self {
            inner: DateTime::from_naive_utc_and_offset(NaiveDateTime::new(date,time), Utc),
        })
    }
}

impl UserData for ChronoWrapper {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("year", |_, this, ()| Ok(this.inner.year()));
        methods.add_method("month", |_, this, ()| Ok(this.inner.month()));
        methods.add_method("day", |_, this, ()| Ok(this.inner.day()));
        methods.add_method("hour", |_, this, ()| Ok(this.inner.hour()));
        methods.add_method("minute", |_, this, ()| Ok(this.inner.minute()));
        methods.add_method("second", |_, this, ()| Ok(this.inner.second()));
        methods.add_method("seconds_float", |_, this, ()| {
            Ok(this.inner.timestamp() as f64 + this.inner.timestamp_subsec_nanos() as f64 / 1_000_000_000.0)
        });
        methods.add_method("millis", |_, this, ()| Ok(this.inner.timestamp_millis()));
        methods.add_method("millisfloat", |_, this, ()| Ok(this.inner.timestamp_millis() as f64));
        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(this.inner.to_rfc3339())
        });
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

impl LuaSetterArg for ChronoWrapper {
    type FromLuaKind = UserDataRef<Self>;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = <Self as LuaSetterArg>::from_lua(arg);
    }
    fn from_lua(arg: Self::FromLuaKind) -> Self {
        (*arg).clone()
    }
}

#[cfg(feature = "serde")]
pub mod rfc3339 {
    use std::{
        borrow::Cow,
        str::FromStr,
    };

    use super::{ChronoWrapper};
    use chrono::{DateTime,Utc};

    #[cfg(feature="serde")]
    use serde::{de::{Deserializer,Deserialize},ser::{Serialize,Serializer}};

    pub fn deserialize<'de,D>(d: D) -> Result<ChronoWrapper,D::Error>
    where
        D: Deserializer<'de>,
    {
        let x = Cow::<'de,str>::deserialize(d)?;
        let dt = DateTime::<Utc>::from_str(&x)
            .map_err(<D::Error as serde::de::Error>::custom)?;
        Ok(ChronoWrapper { inner: dt }) 
    }

    pub fn serialize<S>(this: &ChronoWrapper, s: S) -> Result<S::Ok,S::Error>
    where
        S: Serializer,
    {
        this.inner.serialize(s)
    }
}
