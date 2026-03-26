use crate::adapters::{
    MaybeFuture,
};

pub trait ServiceConfig {

    fn can_initialize(&self) -> MaybeFuture<bool>;
    fn initialize(&self) -> MaybeFuture<Result<(),anyhow::Error>>;

    fn creates_service<'a>(&'a self) -> anyhow::Result<Vec<&'a str>>;
    fn requires_services<'a>(&'a self) -> anyhow::Result<Vec<&'a str>>;
}
