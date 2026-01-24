use std::{
    error::Error,
    any::Any,
};
use mirror_mirror::Reflect;

pub trait Err: 'static + Send + Sync + Error {

    /// Add additional context to an existing error
    fn add_context<const N: usize>(self, msg: Option<std::fmt::Arguments<'_>>, zargs: [(&'static str, &dyn std::fmt::Debug);N]) -> Self
    where
        Self: Sized;

    fn not_an_http_client<T: Any + ?Sized>() -> Self
    where
        Self: Sized;

    fn not_an_http_server<T: Any + ?Sized>() -> Self
    where
        Self: Sized;

    /// Generate a type error, effectively we expected `A` but recieved `B`
    fn type_error<A: Reflect, B: Reflect>(args: Option<std::fmt::Arguments<'_>>) -> Self
    where
        Self: Sized;

    /// For when Registered service lookups fail.
    fn no_such_service(path: &[&str]) -> Self
    where
        Self: Sized;
}
