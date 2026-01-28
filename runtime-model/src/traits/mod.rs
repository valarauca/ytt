
pub mod errors;

pub use self::errors::{Err};

/// Generalized configuration
pub type BoxedConfig = Box<dyn std::any::Any +'static + Send + Send>;

