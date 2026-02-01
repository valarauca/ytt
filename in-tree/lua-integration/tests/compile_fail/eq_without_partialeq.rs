use lua_integration::LuaIntegration;

#[derive(LuaIntegration, Clone)]
#[lua(Eq)]
pub struct ShouldFail {
    pub field: i32,
}

fn main() {}
