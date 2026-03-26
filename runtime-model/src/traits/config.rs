
use std::{
    any::{Any},
};

trait CloneAny: std::any::Any + Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn CloneAny>;
}
impl<T> CloneAny for T
where
    T: std::any::Any + Send + Sync + 'static + Clone,
{
    fn clone_box(&self) -> Box<dyn CloneAny> {
        Box::new(self.clone())
    }
}
