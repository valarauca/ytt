
pub mod errors;

pub use self::errors::{
    not_an_http_client,
    not_an_http_server,
    type_error,
    no_such_service,
    service_has_stopped,
};

pub type BoxedConfig = Box<dyn std::any::Any +'static + Send + Send>;

