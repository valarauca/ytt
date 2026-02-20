use std::{
    collections::{BTreeSet,BTreeMap,HashMap,HashSet},
    hash::{BuildHasher,Hash},
    path::{PathBuf},
    ffi::{CString,OsString},
};

pub trait LuaSetterArg: Clone {
    type FromLuaKind: mlua::FromLua;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind);
    fn from_lua(arg: Self::FromLuaKind) -> Self;
}

macro_rules! boilerplate {
    ($($kind: ty),* $(,)*) => {
        $(
            impl LuaSetterArg for $kind {
                type FromLuaKind = $kind;
                fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
                    *self = arg;
                }
                fn from_lua(arg: Self::FromLuaKind) -> Self { arg }
            }
            impl<const N: usize> LuaSetterArg for [$kind;N] {
                type FromLuaKind = [$kind;N];
                fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
                    *self = arg;
                }
                fn from_lua(arg: Self::FromLuaKind) -> Self { arg }
            }
        )*
    };
}

boilerplate! {
    bool,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    f32, f64,
    CString,
    String,
    OsString,
    PathBuf,
}

impl<A: LuaSetterArg + Ord> LuaSetterArg for BTreeSet<A>
where
    <A as LuaSetterArg>::FromLuaKind: Ord,
    BTreeSet<<A as LuaSetterArg>::FromLuaKind>: mlua::FromLua,
{
    type FromLuaKind = BTreeSet<<A as LuaSetterArg>::FromLuaKind>;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = <Self as LuaSetterArg>::from_lua(arg);
    }
    fn from_lua(arg: Self::FromLuaKind) -> Self {
        arg.into_iter().map(<A as LuaSetterArg>::from_lua).collect()
    }
}
impl<K, V> LuaSetterArg for BTreeMap<K, V>
where
    K: LuaSetterArg + Ord,
    V: LuaSetterArg,
    <K as LuaSetterArg>::FromLuaKind: Ord,
    BTreeMap<<K as LuaSetterArg>::FromLuaKind, <V as LuaSetterArg>::FromLuaKind>: mlua::FromLua,
{
    type FromLuaKind = BTreeMap<<K as LuaSetterArg>::FromLuaKind, <V as LuaSetterArg>::FromLuaKind>;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = <Self as LuaSetterArg>::from_lua(arg);
    }
    fn from_lua(arg: Self::FromLuaKind) -> Self {
        arg.into_iter()
            .map(|(k, v)| (<K as LuaSetterArg>::from_lua(k), <V as LuaSetterArg>::from_lua(v)))
            .collect()
    }
}
impl<K, V, S> LuaSetterArg for HashMap<K, V, S>
where
    K: LuaSetterArg + Eq + Hash,
    V: LuaSetterArg,
    S: BuildHasher + Default + Clone,
    <K as LuaSetterArg>::FromLuaKind: Eq + Hash,
    HashMap<<K as LuaSetterArg>::FromLuaKind, <V as LuaSetterArg>::FromLuaKind>: mlua::FromLua,
{
    type FromLuaKind = HashMap<<K as LuaSetterArg>::FromLuaKind, <V as LuaSetterArg>::FromLuaKind>;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = <Self as LuaSetterArg>::from_lua(arg);
    }
    fn from_lua(arg: Self::FromLuaKind) -> Self {
        let mut out = HashMap::with_hasher(S::default());
        for (k, v) in arg {
            out.insert(
                <K as LuaSetterArg>::from_lua(k),
                <V as LuaSetterArg>::from_lua(v),
            );
        }
        out
    }
}
impl<A, S> LuaSetterArg for HashSet<A, S>
where
    A: LuaSetterArg + Eq + Hash,
    S: BuildHasher + Default + Clone,
    <A as LuaSetterArg>::FromLuaKind: Eq + Hash,
    HashSet<<A as LuaSetterArg>::FromLuaKind>: mlua::FromLua,
{
    type FromLuaKind = HashSet<<A as LuaSetterArg>::FromLuaKind>;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = <Self as LuaSetterArg>::from_lua(arg);
    }
    fn from_lua(arg: Self::FromLuaKind) -> Self {
        let mut out = HashSet::with_hasher(S::default());
        for a in arg {
            out.insert(<A as LuaSetterArg>::from_lua(a));
        }
        out
    }
}
impl<T: LuaSetterArg> LuaSetterArg for Option<T> {
    type FromLuaKind = Option< <T as LuaSetterArg>::FromLuaKind>;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = <Self as LuaSetterArg>::from_lua(arg);
    }
    fn from_lua(arg: Self::FromLuaKind) -> Self {
        arg.map(|x: <T as LuaSetterArg>::FromLuaKind | -> T { <T as LuaSetterArg>::from_lua(x) } )
    }
}
impl<T: LuaSetterArg> LuaSetterArg for Vec<T> {
    type FromLuaKind = Vec< <T as LuaSetterArg>::FromLuaKind>;
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = <Self as LuaSetterArg>::from_lua(arg);
    }
    fn from_lua(arg: Self::FromLuaKind) -> Self {
        // becomes a no-op when the types identical
        arg.into_iter().map(<T as LuaSetterArg>::from_lua).collect()
    }
}

/// A trivial wrapper around serde Json
#[cfg(feature = "serde")]
#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub struct JsonValue(pub serde_json::Value);
#[cfg(feature = "serde")]
impl std::ops::Deref for JsonValue {
    type Target = serde_json::Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for JsonValue {
    fn deserialize<D>(d: D) -> Result<Self,D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Self::from(<serde_json::Value as serde::de::Deserialize>::deserialize::<D>(d)?))
    }
}
#[cfg(feature = "serde")]
impl serde::ser::Serialize for JsonValue {
    fn serialize<S>(&self, s: S) -> Result<S::Ok,S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(s)
    }
}
#[cfg(feature = "serde")]
impl From<serde_json::Value> for JsonValue {
    fn from(x: serde_json::Value) -> JsonValue {
        Self(x)
    }
}
#[cfg(feature = "serde")]
impl LuaSetterArg for JsonValue {
    type FromLuaKind = mlua::Value;
    fn from_lua(arg: mlua::Value) -> Self {
        let de = mlua::serde::Deserializer::new(arg);
        let value: serde_json::Value = <serde_json::Value as serde::Deserialize>::deserialize(de)
            .unwrap_or(serde_json::Value::Null);
        Self::from(value)
    }
    fn set_from_lua(&mut self, arg: Self::FromLuaKind) {
        *self = Self::from_lua(arg);
    }
}
#[cfg(feature = "serde")]
impl mlua::IntoLua for JsonValue {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        use mlua::serde::Serializer;
        use serde::ser::Serialize;
        < serde_json::Value as Serialize>::serialize(&self.0, Serializer::new(lua))
    }
}
#[cfg(feature = "serde")]
impl mlua::FromLua for JsonValue {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        let de = mlua::serde::Deserializer::new(value);
        let value = <serde_json::Value as serde::Deserialize>::deserialize(de)
            .unwrap_or(serde_json::Value::Null);
        Ok(Self::from(value))
    }
}
