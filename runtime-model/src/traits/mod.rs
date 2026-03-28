
pub mod errors;
pub mod config;

pub use self::errors::{
    not_an_http_client,
    not_an_http_server,
    type_error,
    no_such_service,
    service_has_stopped,
    not_a_lua_runtime,
};

pub type BoxedConfig = Box<dyn std::any::Any +'static + Send + Send>;


