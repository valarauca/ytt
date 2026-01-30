use std::{
    error::Error,
    any::Any,
};

pub trait Err: Clone + 'static + Send + Sync + Error
    + From<url::ParseError>
    + From<reqwest::Error>
    + From<reloadable::ReloadableServiceError<reqwest::Error>>
    + From<openrouter::Error>
    + From<serde_json::Error>
{

    /// Add additional context to an existing error
    fn add_context<const N: usize>(self, msg: Option<std::fmt::Arguments<'_>>, zargs: [(&'static str, &dyn std::fmt::Debug);N]) -> Self
    where
        Self: Sized;

    fn from_err<const N: usize>(arg: &dyn Error, msg: &'static str, zargs: [(&'static str, &dyn std::fmt::Debug);N]) -> Self
    where
        Self: Sized;

    fn not_an_http_client<T: Any + ?Sized>() -> Self
    where
        Self: Sized;

    fn not_an_http_server<T: Any + ?Sized>() -> Self
    where
        Self: Sized;

    /// Generate a type error, effectively we expected `A`.
    fn type_error<A: Any>() -> Self
    where
        Self: Sized;

    /// For when Registered service lookups fail.
    fn no_such_service(path: &[&str]) -> Self
    where
        Self: Sized;

    fn service_has_stopped(name: &'static str) -> Self
    where
        Self: Sized;
}
