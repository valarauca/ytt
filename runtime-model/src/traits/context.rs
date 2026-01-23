
use std::time::SystemTime;
use uuid::{Uuid};
use crate::{
    primatives::{Decimal},
};

pub trait ContextBearer: 'static + Send {
    fn initialize_context(&mut self, ctx: Box<dyn Context>);

    fn get_ctx<'a>(&'a self) -> Option<&'a dyn Context>;
    fn get_mut<'a>(&'a mut self) -> Option<&'a mut dyn Context>;
}

/// Contains metadata about this type
pub trait Context: 'static + Send + Sync {
    fn get_user_id(&self) -> Uuid;
    fn get_request_id(&self) -> Uuid;
    fn get_parent_id(&self) -> Option<Uuid>;
    fn consume_budget(&mut self, cost: Decimal);

    fn get_remaining_budget(&self) -> Option<Decimal>;
    fn get_deadline(&self) -> Option<SystemTime>;
}

