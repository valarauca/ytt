//! Derive macros for mlua UserData implementations.
//!
//! Provides two derive macros:
//! - `LuaIntegration` for structs
//! - `LuaKind` for enums
//!
//! All types must implement Clone. See individual macro documentation for details.

pub use lua_integration_derive::{LuaIntegration, LuaKind};

pub use mlua;

#[cfg(feature = "serde")]
pub use serde;

#[cfg(feature = "serde")]
pub use serde_json;

pub mod traits;

#[cfg(feature = "serde")]
pub use self::traits::{JsonValue};

pub mod chrono_impl;
