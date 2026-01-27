use std::{
    task::{Poll, Context, Waker},
    pin::Pin,
    future::{Future,ready,Ready},
    marker::PhantomData,
};

use tokio::sync::watch::{self, Sender};
use tower::{Service,ServiceExt,MakeService};
use futures_util::future::{FutureExt,Either};


use crate::{
    channel::{Channel},
    reloadable::{ReloadableService,InternalState},
};

pub struct ReloadingInstance<C, F, M>
where
    C: Clone + PartialEq + 'static,
    F: Service<C, Response = M>,
    M: Clone + Send + Sync + 'static,
{
    pub(crate) config: C,
    pub(crate) sender: Sender<Result<M, ()>>,
    pub(crate) factory: F,
}
impl<C, F, M> ReloadingInstance<C, F, M>
where
    C: Clone + PartialEq + 'static,
    F: Service<C, Response = M>,
    M: Clone + Send + Sync + 'static,
{
    /// Create a new ReloadingInstance with initial configuration
    pub async fn new(config: C, mut factory: F) -> Result<Self, F::Error> {
        factory.ready().await?;
        let service = factory.call(config.clone()).await?;
        let (sender, _receiver) = watch::channel(Ok(service));
        
        Ok(Self { 
            config,
            sender, 
            factory,
        })
    }

    /// Return an instance to the underlying service
    pub fn get_service_handle<R,S>(&self) -> ReloadableService<M,S,R>
    where
        R: Send + 'static,
        S: Service<R> + Send + 'static,
        M: Service<(),Response=S> + MakeService<(),R> + Clone + Send + Sync + 'static,
        <S as Service<R>>::Response: Send + 'static,
        <S as Service<R>>::Future: Send + 'static,
        <S as Service<R>>::Error: Send + 'static,
        <M as Service<()>>::Response: Send + 'static,
        <M as Service<()>>::Error: Send + 'static,
        <M as Service<()>>::Future: Send + 'static,
    {
        let reload = Channel::from(self.sender.clone());
        ReloadableService {
            status: InternalState::Uninitialized,
            reload: reload,
            _marker_service: PhantomData,
            _marker_request: PhantomData,
        }
    }

    pub fn reload(&mut self, config: C) -> Either<Ready<Result<(),F::Error>>,Pin<Box<dyn Future<Output=Result<(),F::Error>> + 'static + Send>>>
    where
        C: Send + 'static,
        F: Service<C, Response = M> + Clone + Send + 'static,
        F::Future: Unpin + Send + 'static,
        F::Error: Send + 'static,
    {
        if self.config.eq(&config) {
            return ready(Ok(())).left_future();
        }
        self.config.clone_from(&config);
        let _ = self.sender.send_replace(Err(()));
        let mut fake = Context::from_waker(&Waker::noop());
        let result_ready = match self.factory.poll_ready(&mut fake) {
            Poll::Pending => {
                let tx = self.sender.clone();
                let mut f = self.factory.clone();
                let future: Pin<Box<dyn Future<Output=Result<(),F::Error>> + 'static + Send>> = Box::pin(async move {
                    f.ready().await?;
                    let out = f.call(config).await?;
                    let _ = tx.send_replace(Ok(out));
                    Ok(())
                });
                return future.right_future();
            }
            Poll::Ready(res) => res,
        };
        match result_ready {
            Err(e) => {
                return ready(Err(e)).left_future();
            }
            Ok(()) => { }
        };
        let mut future = self.factory.call(config);
        match Pin::new(&mut future).poll(&mut fake) {
            Poll::Ready(Err(e)) => {
                return ready(Err(e)).left_future();
            }
            Poll::Ready(Ok(x)) => {
                let _ = self.sender.send_replace(Ok(x));
                // fully synchronous update lmaooo
                return ready(Ok(())).left_future();
            }
            Poll::Pending => {
                let tx = self.sender.clone();
                let boxed: Pin<Box<dyn Future<Output=Result<(),F::Error>> + 'static + Send>> = Box::pin(async move {
                    let out = future.await?;
                    let _ = tx.send_replace(Ok(out));
                    Ok(())
                });
                return boxed.right_future();
            }
        };
    }
}
