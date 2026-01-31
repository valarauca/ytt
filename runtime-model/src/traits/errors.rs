use std::any::Any;

pub fn not_an_http_client<T: Any + ?Sized>() -> anyhow::Error {
    anyhow::anyhow!(
        "Expected HTTP client, got: {}",
        std::any::type_name::<T>()
    )
}

pub fn not_an_http_server<T: Any + ?Sized>() -> anyhow::Error {
    anyhow::anyhow!(
        "Expected HTTP server, got: {}",
        std::any::type_name::<T>()
    )
}

pub fn type_error<A: Any>() -> anyhow::Error {
    anyhow::anyhow!(
        "Type error: expected {}",
        std::any::type_name::<A>()
    )
}

pub fn no_such_service(path: &[&str]) -> anyhow::Error {
    anyhow::anyhow!(
        "No service found at path: {}",
        path.join("/")
    )
}

pub fn service_has_stopped(name: &'static str) -> anyhow::Error {
    anyhow::anyhow!(
        "Service has stopped: {}",
        name
    )
}

