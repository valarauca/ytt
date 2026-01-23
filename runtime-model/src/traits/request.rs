
use super::context::{ContextBearer};

pub trait Request: 'static + Send + ContextBearer {
    
}
