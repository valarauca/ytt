use lua_integration::LuaKind;

#[derive(LuaKind)]
pub struct ShouldFail {
    field: i32,
}

fn main() {}
