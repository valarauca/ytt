use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod structs;
mod enums;
mod attrs;

/// Derives mlua::UserData for structs.
///
/// Generates field accessors for public named fields. Private fields are not exposed to Lua.
/// Tuple structs and unit structs are not supported.
///
/// Type must implement Clone. Field types must implement Clone.
///
/// # Attributes
///
/// - `#[lua(serde)]` - Adds ToString metamethod (JSON) and from_json function. Requires serde feature.
/// - `#[lua(Eq)]` - Adds Eq metamethod. Type must implement PartialEq.
///
/// # Example
///
/// ```ignore
/// #[derive(LuaIntegration, Clone)]
/// struct Point {
///     pub x: f64,
///     pub y: f64,
/// }
/// ```
#[proc_macro_derive(LuaIntegration, attributes(lua))]
pub fn derive_lua_integration(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(_) => structs::expand_struct(input),
        syn::Data::Enum(_) => {
            syn::Error::new_spanned(
                input,
                "LuaIntegration does not support enums. Use LuaKind instead."
            )
            .to_compile_error()
            .into()
        }
        syn::Data::Union(_) => {
            syn::Error::new_spanned(
                input,
                "LuaIntegration does not support unions"
            )
            .to_compile_error()
            .into()
        }
    }
}

/// Derives mlua::UserData for enums.
///
/// Generates methods for variant checking and field access:
/// - `is_{variant}()` returns bool
/// - `get_{variant}()` returns Option for single-field tuple variants
/// - `get_{variant}_{index}` for multi-field tuple variants
/// - `get_{variant}_{field}` for struct variants
///
/// Type must implement Clone. Field types must implement Clone.
///
/// # Attributes
///
/// Container attributes:
/// - `#[lua(serde)]` - Adds ToString metamethod (JSON) and from_json function. Requires serde feature.
///
/// Variant attributes:
/// - `#[lua(rename = "name")]` - Changes Lua-visible variant name.
///
/// # Example
///
/// ```ignore
/// #[derive(LuaKind, Clone)]
/// enum Status {
///     Idle,
///     Running(u32),
///     Error { code: i32 },
/// }
/// ```
#[proc_macro_derive(LuaKind, attributes(lua))]
pub fn derive_lua_kind(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Enum(_) => enums::expand_enum(input),
        syn::Data::Struct(_) => {
            syn::Error::new_spanned(
                input,
                "LuaKind is only for enums. Use LuaIntegration for structs."
            )
            .to_compile_error()
            .into()
        }
        syn::Data::Union(_) => {
            syn::Error::new_spanned(
                input,
                "LuaKind does not support unions"
            )
            .to_compile_error()
            .into()
        }
    }
}
