use std::{
    pin::{Pin},
    future::Future,
    task::{Context,Poll},
    sync::{Arc},
};
use tokio::{
    sync::{Mutex},
};
use crate::{
    traits::{BoxedConfig},
};
use super::config::{HttpServerConfig,HttpListener,ListenerFuture};



pub struct ServiceWrapper {
    inner: Arc<Mutex<Option<ListenerFuture>>>,
}
impl tower::Service<()> for ServiceWrapper {
    type Response = Option<ListenerFuture>;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            Ok(inner.lock_owned().await.take())
        })
    }
}
