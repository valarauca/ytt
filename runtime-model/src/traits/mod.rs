
pub mod errors;
pub mod registered;
//pub mod request;
//pub mod response;
//pub mod context;

pub use self::errors::{Err};

/// Generalized configuration
pub type BoxedConfig = Box<dyn std::any::Any +'static + Send + Send>;

