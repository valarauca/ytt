use std::{
    task::{Poll, Context},
    pin::Pin,
    future::{Future,ready,Ready},
};

use tokio::sync::watch::{self, Sender};
use tower::{Service,ServiceExt};
use futures_util::future::{FutureExt,Either};


use crate::{
    channel::Channel,
    reloadable::ReloadableService,
};

pub struct ReloadingInstance<C, S, F>
where
    S: Clone + Send + Sync + 'static,
    C: Clone + PartialEq + 'static,
    F: Service<C, Response = S>,
{
    pub(crate) config: C,
    pub(crate) sender: Sender<Result<S, ()>>,
    pub(crate) factory: F,
}
impl<C, S, F> ReloadingInstance<C, S, F>
where
    S: Clone + Send + Sync + 'static,
    C: Clone + PartialEq + 'static,
    F: Service<C, Response = S>,
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
    
    /// Get a Channel for subscribing to service updates
    pub(crate) fn channel(&self) -> Channel<S> {
        Channel::from(self.sender.clone())
    }
    
    /// Create a ReloadableService instance
    pub fn service<E, R>(&self) -> ReloadableService<E, R, S>
    where
        S: Service<R> + 'static,
        E: Send + 'static,
    {
        ReloadableService::new(self.channel())
    }
}

impl<C, S, F> Service<C> for ReloadingInstance<C, S, F>
where
    C: Clone + PartialEq + Send + 'static,
    S: Clone + Send + Sync + 'static,
    F: Service<C, Response = S> + Clone + Send + 'static,
    F::Future: Send + 'static,
    F::Error: Send + 'static,
{
    type Response = ();
    type Error = F::Error;
    type Future = Either<Ready<Result<(),Self::Error>>,Pin<Box<dyn Future<Output=Result<(),Self::Error>> + 'static + Send>>>;
    
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    
    fn call(&mut self, req: C) -> Self::Future {
        if req == self.config {
            ready(Result::<(),Self::Error>::Ok(())).left_future()
        } else {
            self.config.clone_from(&req);
            let _ = self.sender.send_replace(Err(()));
            let tx = self.sender.clone();
            let mut f = self.factory.clone();
            let x: Pin<Box<dyn Future<Output=Result<(),Self::Error>> + 'static + Send>> = Box::pin(async move {
                f.ready().await?;
                let out = f.call(req).await?;
                let _ = tx.send_replace(Ok(out));
                Ok(())
            });
            x.right_future()
        }
    }
}
