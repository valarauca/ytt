use std::any::type_name;
use serde::de::{Error,DeserializeOwned};

/// Format describes a 'middleware' format.
///
/// For some data types we need a notion of
///
/// The data gets read in, deserialized as `$Format`, then that is deserialized into `T`.
/// This trait handles that abstraction.
pub trait Format {

    fn deserialize_str<T,E>(_buffer: &str) -> Result<T,E>
    where
        T: DeserializeOwned,
        E: Error,
    {
        Err(E::custom("unsupported"))
    }
}

impl<A: Format,B: Format> Format for (A,B) {
    fn deserialize_str<T,E>(buffer: &str) -> Result<T,E>
    where
        T: DeserializeOwned,
        E: Error,
    {
        let a_err = match A::deserialize_str::<T,E>(buffer) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };
        let b_err = match B::deserialize_str::<T,E>(buffer) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };
        Err(E::custom(format!("failed to deserialize either A: '{}' with error: '{:?}' & B: '{}' with error: '{:?}'", type_name::<A>(), a_err, type_name::<B>(), b_err)))
    }
}
impl<A: Format,B: Format,C: Format> Format for (A,B,C) {
    fn deserialize_str<T,E>(buffer: &str) -> Result<T,E>
    where
        T: DeserializeOwned,
        E: Error,
    {
        let a_err = match A::deserialize_str::<T,E>(buffer) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };
        let b_err = match B::deserialize_str::<T,E>(buffer) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };
        let c_err = match C::deserialize_str::<T,E>(buffer) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };
        Err(E::custom(format!("failed to deserialize either A: '{}' with error: '{:?}' & B: '{}' with error: '{:?}' & C: '{}' with error: '{:?}'", type_name::<A>(), a_err, type_name::<B>(), b_err, type_name::<C>(), c_err)))
    }
}

/// Handle semantics of serializing JSON
#[cfg(feature = "json")]
pub struct Json;
#[cfg(feature = "json")]
impl Format for Json {

    fn deserialize_str<T,E>(buffer: &str) -> Result<T,E> 
    where
        T: DeserializeOwned,
        E: Error,
    {
        serde_json::from_str(buffer)
            .map_err(|e| E::custom(e))
    }
}

/// Handle semantics related to deserializing a JSON5 buffer
#[cfg(feature = "json5")]
pub struct Json5;
#[cfg(feature = "json5")]
impl Format for Json5 {
    fn deserialize_str<T,E>(buffer: &str) -> Result<T,E> 
    where
        T: DeserializeOwned,
        E: Error,
    {
        json5::from_str(buffer)
            .map_err(|e| E::custom(e))
    }
}

#[cfg(feature = "toml")]
pub struct Toml;
#[cfg(feature = "toml")]
impl Format for Toml {
    fn deserialize_str<T,E>(buffer: &str) -> Result<T,E> 
    where
        T: DeserializeOwned,
        E: Error,
    {
        toml::from_str::<T>(buffer)
            .map_err(|e| E::custom(e))
    }
}

#[cfg(feature = "yaml")]
pub struct Yaml;
#[cfg(feature = "yaml")]
impl Format for Yaml {
    fn deserialize_str<T,E>(buffer: &str) -> Result<T,E> 
    where
        T: DeserializeOwned,
        E: Error,
    {
        serde_yaml::from_str::<T>(buffer)
            .map_err(|e| E::custom(e))
    }
}

#[cfg(feature = "ron")]
pub struct Ron;
#[cfg(feature = "ron")]
impl Format for Ron {
    fn deserialize_str<T,E>(buffer: &str) -> Result<T,E> 
    where
        T: DeserializeOwned,
        E: Error,
    {
        ron::from_str::<T>(buffer)
            .map_err(|e| E::custom(e))
    }
}


