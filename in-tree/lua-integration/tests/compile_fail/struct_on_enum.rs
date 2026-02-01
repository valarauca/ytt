use lua_integration::LuaIntegration;

#[derive(LuaIntegration)]
pub enum ShouldFail {
    Variant,
}

fn main() {}
