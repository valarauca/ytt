use std::{
    task::{Poll, Context},
};

use tokio::sync::watch::{self, Sender};
use tower::{Service,ServiceExt};
use futures_util::future::{MapOk, TryFutureExt};

use mirror_mirror::{Reflect};
use mirror_mirror_opinions::update_reflect;

use crate::{
    channel::Channel,
    reloadable::ReloadableService,
};

pub struct ReloadingInstance<C, S, F>
where
    S: Clone + Send + Sync + 'static,
    C: Clone + Reflect + 'static,
    F: Service<C, Response = S>,
{
    pub(crate) config: C,
    pub(crate) sender: Sender<Result<S, ()>>,
    pub(crate) factory: F,
    pub(crate) paused: bool,
}
impl<C, S, F> ReloadingInstance<C, S, F>
where
    S: Clone + Send + Sync + 'static,
    C: Clone + Reflect + 'static,
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
            paused: false,
        })
    }
    
    /// Get a Channel for subscribing to service updates
    pub fn channel(&self) -> Channel<S> {
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
    C: Clone + Reflect + Send + 'static,
    S: Clone + Send + Sync + 'static,
    F: Service<C, Response = S> + Send + 'static,
    F::Future: Send + 'static,
    F::Error: Send + 'static,
{
    type Response = ();
    type Error = F::Error;
    type Future = MapOk<F::Future, Box<dyn FnOnce(S) -> () + Send>>;
    
    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if !self.paused {
			// kick clients into `Uninitialized` status
            let _ = self.sender.send(Err(()));
            self.paused = true;
        }
        self.factory.poll_ready(ctx)
    }
    
    fn call(&mut self, req: C) -> Self::Future {
        self.paused = false;
        let sender = self.sender.clone();
        let lambda: Box<dyn FnOnce(S) -> () + Send> = Box::new(move |arg: S| -> () {
            let _ = sender.send(Ok(arg));
            ()
        });

        // errors can only occur if our config type can change
        // which it shouldn't as our stuff is statically compiled.
        update_reflect(&mut self.config, &req).unwrap();
        self.factory.call(self.config.clone()).map_ok(lambda)
    }
}
