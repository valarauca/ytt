
use std::{
    error::Error,
    any::Any,
};

pub trait Err: 'static + Send + Sync + Error {

    fn not_an_http_client<T: Any + ?Sized>() -> Self
    where
        Self: Sized;

    fn not_an_http_server<T: Any + ?Sized>() -> Self
    where
        Self: Sized;
}
