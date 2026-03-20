use crate::adapters::{
    MaybeFuture,
};

pub trait ServiceConfig {
    fn can_initialize(&self) -> MaybeFuture<bool>;
    fn initialize(&self) -> MaybeFuture<Result<(),anyhow::Error>>;
}
