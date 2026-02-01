# lua-integration

Derive macros for generating mlua UserData implementations.

## Usage

Add to Cargo.toml:
```toml
[dependencies]
lua-integration = { path = "./lua-integration" }
```

## Derive Macros

### LuaIntegration

Implements `mlua::UserData` for structs. Only named fields are supported. Public fields generate getters and setters accessible from Lua.

```rust
use lua_integration::LuaIntegration;

#[derive(LuaIntegration, Clone)]
struct Point {
    pub x: f64,
    pub y: f64,
}
```

Lua access:
```lua
point.x = 10.0
local value = point.y
```

### LuaKind

Implements `mlua::UserData` for enums. Generates methods for variant checking and field access.

```rust
use lua_integration::LuaKind;

#[derive(LuaKind, Clone)]
enum Status {
    Idle,
    Running(u32),
    Error { code: i32, message: String },
}
```

Generated methods:
- `is_idle()` - returns bool
- `get_running()` - returns Option for single-field tuple variants
- `get_error_code()` - returns Option for struct variant fields

## Attributes

### Container Attributes

- `#[lua(serde)]` - Generates `ToString` metamethod (JSON) and `from_json` function. Requires `serde` feature and type must implement Serialize/Deserialize.
- `#[lua(Eq)]` - Generates equality metamethod. Type must implement PartialEq.

### Variant Attributes

- `#[lua(rename = "name")]` - Changes the Lua-visible name of an enum variant.

## Requirements

All types using these derives must implement `Clone`. Field types must also implement `Clone` as accessors return cloned values.

## Features

- `serde` - Enables serde integration for JSON serialization.
