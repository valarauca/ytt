/*use std::{
    sync::{Arc,atomic::Ordering},
};
*/
use sdd::{AtomicShared,Guard};
use tower::{Service};

#[derive(Clone)]
pub struct SharedService<S> {
    data: AtomicShared<S>,
}
impl<Req> Service<Req> for SharedService<S>
where
    S: Service<Req>,
{
    type Response = <S as Service<Req>>::Response;
    type Error = <S as Service<Req>>::Error;
    type Future = <S as Service<Req>>::Future;
    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        let mut g = Guard::new();
        self.data.load(Ordering::Acquire,&g).as_ref().unwrap().po
    }
}
