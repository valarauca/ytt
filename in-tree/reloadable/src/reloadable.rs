use std::{
    pin::Pin,
    task::{Poll,Context,Waker},
    marker::PhantomData,
    error::Error,
};

use futures_util::{
    stream::TryStream,
    future::{
        MapErr,TryFutureExt,
    },
};
use tower::Service;

use super::channel::{Channel};


pub(crate) enum InternalState<S> {
    Uninitialized,
    Normal(S),
    Finished,
}
impl<S> InternalState<S> {

    fn as_normal<'a>(&'a mut self) -> Option<&'a mut S> {
        match self {
            &mut Self::Normal(ref mut s) => Some(s),
            _ => None
        }
    }

    fn is_uninitialized(&self) -> bool {
        match self {
            Self::Uninitialized => true,
            _ => false,
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::Finished => true,
            _ => false,
        }
    }

    fn is_normal(&self) -> bool {
        match self {
            Self::Normal(_) => true,
            _ => false,
        }
    }
}

pub type SendTryStream<S,E> = Pin<Box<dyn TryStream<Ok=S,Error=E,Item=Result<S,E>> + Send + 'static>>;


/// Generalized Error Type
pub trait Err: Error {
    fn stopped() -> Self
    where
        Self: Sized;
}

pub struct ReloadableService<E,R,S> 
where
    S: Send + 'static,
    E: Send + 'static,
{
    pub(crate) status: InternalState<S>,
    _marker_request: PhantomData<fn(R)>,
    _marker_error: PhantomData<fn(E)>,
    reload: SendTryStream<S,()>,
}
impl<E,R,S> ReloadableService<E,R,S>
where
    S: Service<R> + Clone + Send + Sync + 'static,
    E: Send + 'static,
{

    /// Create a non-initialized service
    pub fn new(channel: Channel<S>) -> Self {
        Self {
            status: InternalState::Uninitialized,
            reload: Box::pin(channel),
            _marker_request: PhantomData,
            _marker_error: PhantomData,
        }
    }
}

impl<E,R,S> Service<R> for ReloadableService<E,R,S>
where
    S: Service<R> + Send + 'static,
    E: Sized + Clone + From<<S as Service<R>>::Error> + Err + Send + 'static,
    <S as Service<R>>::Response: Send + 'static,
    <S as Service<R>>::Future: Send + 'static,
{
    type Response = <S as Service<R>>::Response;
    type Error = E;
    type Future = MapErr<<S as Service<R>>::Future,fn(<S as Service<R>>::Error) -> E>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        loop {
            if self.status.is_normal() {
                // check our service with a noop waker
                //
                // this is because we do **NOT** want
                // to block on it. We presently have
                // a service that "should" work.
                //
                // so we're just validating we have the latest
                // version
                let mut fake = Context::from_waker(&Waker::noop());
                match self.reload.as_mut().try_poll_next(&mut fake) {
                    Poll::Pending => {
                        // IMPORTANT:
                        //
                        // Our service is in a normal state
                        // we're just checking if the management
                        // has an update for us.
                        //
                        // CRITICAL: fallthroughs
                    }
                    Poll::Ready(None) => {
                        self.status = InternalState::Finished;
                        return Poll::Ready(Err(E::stopped()));
                    }
                    Poll::Ready(Some(Err(()))) => {
                        self.status = InternalState::Uninitialized;
                        continue;
                    }
                    Poll::Ready(Some(Ok(s))) => {
                        self.status = InternalState::Normal(s);
                        // CRITICAL: fallthroughs
                    }
                };
                assert!(self.status.is_normal());
                match self.status.as_normal().unwrap().poll_ready(ctx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Ok(())) => return Poll::Ready(Ok(())),
                    Poll::Ready(Err(_)) => {
                        // TODO: log this
                        self.status = InternalState::Uninitialized;
                        // loop back so our context is setup
                        // to monitor the stream
                        continue;
                    }
                };
            }

            if self.status.is_finished() {
                return Poll::Ready(Err(E::stopped()));
            }

            if self.status.is_uninitialized() {
                // poll directly on our reload stream
                //
                // Thusly if we are pending, we will be correctly
                // woken up when something occurs.
                match self.reload.as_mut().try_poll_next(ctx) {
                    Poll::Pending => {
                        return Poll::Pending;
                    },
                    Poll::Ready(None) => {
                        self.status = InternalState::Finished;
                        return Poll::Ready(Err(E::stopped()));
                    },
                    Poll::Ready(Some(Err(()))) => {
                        self.status = InternalState::Uninitialized;
                    },
                    Poll::Ready(Some(Ok(s))) => {
                        self.status = InternalState::Normal(s);
                    },
                };
                continue;
            }
        }
    }
    fn call(&mut self, req: R) -> Self::Future {
        assert!(self.status.is_normal(), "always call `is_ready` before calling a service");
        let f: fn(<S as Service<R>>::Error) -> E = <E as From<<S as Service<R>>::Error>>::from;
        self.status.as_normal().unwrap().call(req).map_err(f)
    }
}

