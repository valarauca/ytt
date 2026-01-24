use std::{
    pin::Pin,
    task::{Poll,Context},
    future::Future,
};

use tokio::sync::watch::{Receiver,Sender};
use futures_util::{
    stream::{Stream},
};


pub(crate) enum Status<S> {
    Fine(Pin<Box<dyn Future<Output=Result<(Result<S,()>,Receiver<Result<S,()>>),()>> + Send + 'static>>),
    Terminated,
}
impl<S> Status<S> {
    fn as_fine<'a>(&'a mut self) -> Option<&'a mut Pin<Box<dyn Future<Output=Result<(Result<S,()>,Receiver<Result<S,()>>),()>> + Send + 'static>>> {
        match self {
            &mut Self::Fine(ref mut s) => Some(s),
            _ => None,
        }
    }
    fn has_terminated(&self) -> bool {
        matches!(self, Self::Terminated)
    }
}

pub struct Channel<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) handle: Sender<Result<S,()>>,
    pub(crate) interior: Status<S>,

}
impl<S> From<Sender<Result<S,()>>> for Channel<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn from(handle: Sender<Result<S,()>>) -> Channel<S> {
        let recv = handle.subscribe();
        let curr: Result<S,()> = recv.borrow().clone();
        let interior = Status::Fine(Box::pin(std::future::ready(Ok((curr,recv)))));
        Self { handle, interior }
    }
}
impl<S> Clone for Channel<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self::from(self.handle.clone())
    }
}
impl<S> Stream for Channel<S>
where
    S: Clone + Send + Sync + 'static,
{
    type Item = Result<S,()>;
    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let s: &mut Self = self.get_mut();
        if s.interior.has_terminated() {
            return Poll::Ready(None);
        }
        let (item,recv) = match s.interior.as_fine().unwrap().as_mut().poll(ctx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(())) => {
                s.interior = Status::Terminated;
                return Poll::Ready(None);
            }
            Poll::Ready(Ok(x)) => x,
        };
        s.interior = Status::Fine(Box::pin(async move {
            let mut recv: Receiver<Result<S,()>> = recv;
            match recv.changed().await {
                Ok(()) => {
                    let x: Result<S,()> = recv.borrow_and_update().clone();
                    return Ok((x,recv));
                }
                Err(_) => {
                    return Err(());
                }
            };
        }));
        Poll::Ready(Some(item))
    }
}
