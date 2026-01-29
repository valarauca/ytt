use std::{
    any::Any,
    sync::{Arc},
    pin::{pin,Pin},
    task::{Poll,Context},
};

use tower::{Service,ServiceExt};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::{
    RwLock,
    oneshot::{Sender as OSSend, channel as os_channel},
    mpsc::{Receiver as MPSCRecv, Sender as MPSCSender, channel as mpsc_channel},
};
use futures_util::{
    future::TryFutureExt,
    stream::StreamExt,
};
use crate::{
    traits::{Err,BoxedConfig},
};

pub trait Reconfig<Req,Res,E: Err>: 'static + Send + Sync
where
    Req: Send + 'static,
    Res: Send + 'static,
{
    fn reconfig<'a>(&'a self, _config: BoxedConfig) -> Pin<Box<dyn Future<Output=Result<(),E>> + 'a + Send>>;
    fn get_service(&self) -> tower::util::BoxCloneService<Req,Res,E>;
}


pub struct ReconfigurableService<C,Req,Res,E: Err> {
    tx: MPSCSender<ServiceComms<C,Req,Res,E>>,
    name: &'static str,
    handle: tokio::task::JoinHandle<()>,
}
impl<C, Req, Res, E> Reconfig<Req,Res,E> for ReconfigurableService<C,Req,Res,E>
where
    C: Any + Clone + PartialEq + Sync + Send + 'static,
    E: Err,
    Req: Send + 'static,
    Res: Send + 'static,
{
    fn reconfig<'a>(&'a self, config: BoxedConfig) -> Pin<Box<dyn Future<Output=Result<(),E>> + 'a + Send>> {
        Box::pin(async {
            let x = config.downcast::<C>().map_err(|_| E::type_error::<C>())?;
            self.reconfigure(*x).await
        })
    }

    fn get_service(&self) -> tower::util::BoxCloneService<Req,Res,E> {
        self.make_request_handle().boxed_clone()
    }
}

impl<C,Req,Res,E: Err + Sized> ReconfigurableService<C,Req,Res,E> {

    /// Constrct a new service
    pub fn new<F,S>(
        config: C,
        buffer: usize,
        factory: F,
    ) -> Self
    where
        Req: Send + 'static,
        Res: Send + 'static,
        E: Clone + Send + 'static,
        C: Clone + PartialEq + Sync + Send + 'static,
        F: Service<C,Response=S,Error=E> + Send + 'static,
        S: Service<Req,Response=Res,Error=E> + Send + 'static,
        <S as Service<Req>>::Future: Send + 'static,
        <F as Service<C>>::Future: Send,
    {
        let buffer = buffer.max(1);
        let (tx,recv) = mpsc_channel(buffer);
        let handle = tokio::task::spawn(spawn_service(config, buffer, factory, recv));
        let name = std::any::type_name::<S>();
        Self { tx, handle, name }
    }


    pub async fn request_graceful_stop(self) {
        if self.handle.is_finished() {
            return;
        }
        let _ = self.tx.send(ServiceComms::StopGraceful).await;
        let _ = self.handle.await;
    }

    pub async fn request_immediate_stop(self) {
        if self.handle.is_finished() {
            return;
        }
        let _ = self.tx.send(ServiceComms::StopNow).await;
        let _ = self.handle.await;
    }

    /// Request the service reconfigure itself.
    ///
    /// If the service is stopped this requires `None`
    pub async fn reconfigure(&self, config: C) -> Result<(),E>
    where
        E: Clone + Send + 'static,
        C: Clone + PartialEq + Sync + Send + 'static,
    {
        if self.handle.is_finished() {
            return Err(E::service_has_stopped(self.name));
        }
        let (tx,rx) = os_channel();
        let _ = self.tx.send(ServiceComms::Reconfigure(config)).await;
        let _ = self.tx.send(ServiceComms::Ready(tx)).await;
        match rx.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(E::service_has_stopped(self.name))
        }
    }

    /// Create a handle that will send requests to the underlying service
    pub fn make_request_handle(&self) -> RequestHandle<C,Req,Res,E>
    where
        Req: Send + 'static,
        Res: Send + 'static,
        E: Clone + Send + 'static,
        C: Clone + PartialEq + Sync + Send + 'static,
    {
        RequestHandle { tx: self.tx.clone(), name: self.name }
    }
}

pub struct RequestHandle<C,Req,Res,E> {
    name: &'static str,
    tx: MPSCSender<ServiceComms<C,Req,Res,E>>,
}
impl<C,Req,Res,E> Clone for RequestHandle<C,Req,Res,E> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            tx: self.tx.clone()
        }
    }
}
impl<C,Req,Res,E: Err + Sized> RequestHandle<C,Req,Res,E> {

    /// returns `Err(None)` if the service is shutdown
    pub async fn make_request(&self, req: Req) -> Result<Res,E>
    where
        Req: Send + 'static,
        Res: Send + 'static,
        E: Clone + Send + 'static,
        C: Clone + PartialEq + Sync + Send + 'static,
    {
        let (tx,rx) = os_channel();
        let _ = self.tx.send(ServiceComms::Request(req, tx)).await;
        match rx.await {
            Err(_) => Err(E::service_has_stopped(self.name)),
            Ok(Ok(res)) => Ok(res),
            Ok(Err(e)) => Err(e),
        }
    }
}
impl<C, Req, Res, E> Service<Req> for RequestHandle<C, Req, Res, E>
where
    Req: Send + 'static,
    Res: Send + 'static,
    E: Err + Sized + Clone + Send + 'static,
    C: Clone + PartialEq + Sync + Send + 'static,
{
    type Response = Res;
    type Error = E;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>> + Send + 'static>>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: Req) -> Self::Future {
        let this = self.clone();
        Box::pin(async move { this.make_request(req).await })
    }
}

async fn spawn_service<C,F,S,Req,Res,E>(
    config: C,
    _buffering: usize,
    factory: F,
    channel: MPSCRecv<ServiceComms<C,Req,Res,E>>
)
where
    Req: Send + 'static,
    Res: Send + 'static,
    E: Clone + Send + 'static,
    C: Clone + PartialEq + Sync + Send + 'static,
    F: Service<C,Response=S,Error=E> + Send + 'static,
    S: Service<Req,Response=Res,Error=E> + Send + 'static,
    <S as Service<Req>>::Future: Send + 'static,
    <F as Service<C>>::Future: Send,
{
    let mut factory = factory;
    let mut config = config;
    let mut service: Option<Result<S,E>> = Some(factory.ready().and_then(|future| future.call(config.clone())).await);

    let graceful_stop_request = Arc::new(RwLock::new(true));
    let stoppage = graceful_stop_request.clone();

    let recv_stream = ReceiverStream::new(channel)
        .take_while(move |_| {
            let x = stoppage.clone();
            async move { 
                x.clone().read_owned().await.clone()
            }
        });
    let mut recv_stream = pin!(recv_stream);
    'reload: loop {
        let mut result = service.take().unwrap();
        'work: while let Some(item) = recv_stream.next().await {
            match item {
                ServiceComms::StopNow => {
                    break 'reload;
                },
                ServiceComms::StopGraceful => {
                    // `false` terminates `take_while`
                    let mut w = graceful_stop_request.clone().write_owned().await;
                    *w = false;
                    continue 'reload;
                },
                ServiceComms::Reconfigure(new_config) => {
                    if config == new_config {
                        continue 'work;
                    }
                    config.clone_from(&new_config);
                    service = Some(factory.ready().and_then(|future| future.call(new_config)).await);
                    continue 'reload;
                },
                ServiceComms::Request(req,resp_chan) => {
                    let out = match result.as_mut() {
                        Ok(srvc) => {
                            srvc.ready().and_then(|rdy| rdy.call(req)).await
                        }
                        Err(e) => Err(e.clone()),
                    };
                    let _ = resp_chan.send(out);
                    continue 'work;
                },
                ServiceComms::Ready(resp_chan) => {
                    let out = match result.as_mut() {
                        Ok(srvc) => srvc.ready().await,
                        Err(e) => Err(e.clone()),
                    };
                    let _ = resp_chan.send(out.map(|_| ()));
                }
            };
        }
    }
    return ();
}

enum ServiceComms<C,Req,Res,E> {
    /// Service is being asked to stop
    StopNow,
    /// Stop processing new incoming connections
    /// but finish pending
    StopGraceful,
    /// Service is being asked to reconfigure itself
    Reconfigure(C),
    /// New incoming requests
    Request(Req,OSSend<Result<Res,E>>),
    /// Send Ready check.
    ///
    /// This is not used for traditional `tower::Service` readiness semantics
    /// this is largely used to validated a configuration change worked.
    ///
    /// Normal service calls will call into `poll_ready` prior to processing.
    Ready(OSSend<Result<(),E>>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::{Context, Poll};

    #[derive(Clone, Debug, PartialEq)]
    struct TestConfig {
        multiplier: i32,
    }

    #[derive(Clone)]
    struct TestServiceFactory;

    impl Service<TestConfig> for TestServiceFactory {
        type Response = TestService;
        type Error = String;
        type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, config: TestConfig) -> Self::Future {
            Box::pin(async move {
                Ok(TestService { multiplier: config.multiplier })
            })
        }
    }

    struct TestService {
        multiplier: i32,
    }

    impl Service<i32> for TestService {
        type Response = i32;
        type Error = String;
        type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: i32) -> Self::Future {
            let result = req * self.multiplier;
            Box::pin(async move { Ok(result) })
        }
    }

    #[tokio::test]
    async fn test_basic_request() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config, 10, TestServiceFactory);
        let handle = service.make_request_handle();

        let result = handle.make_request(5).await;
        assert_eq!(result, Some(Ok(10)));

        service.request_immediate_stop().await;
    }

    #[tokio::test]
    async fn test_multiple_requests() {
        let config = TestConfig { multiplier: 3 };
        let service = ReconfigurableService::new(config, 10, TestServiceFactory);
        let handle = service.make_request_handle();

        let result1 = handle.make_request(2).await;
        let result2 = handle.make_request(4).await;
        let result3 = handle.make_request(10).await;

        assert_eq!(result1, Some(Ok(6)));
        assert_eq!(result2, Some(Ok(12)));
        assert_eq!(result3, Some(Ok(30)));

        service.request_immediate_stop().await;
    }

    #[tokio::test]
    async fn test_reconfigure() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config, 10, TestServiceFactory);
        let handle = service.make_request_handle();

        let result1 = handle.make_request(5).await;
        assert_eq!(result1, Some(Ok(10)));

        let new_config = TestConfig { multiplier: 5 };
        let reconfig_result = service.reconfigure(new_config).await;
        assert_eq!(reconfig_result, Some(Ok(())));

        let result2 = handle.make_request(5).await;
        assert_eq!(result2, Some(Ok(25)));

        service.request_immediate_stop().await;
    }

    #[tokio::test]
    async fn test_reconfigure_same_config() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config.clone(), 10, TestServiceFactory);
        let handle = service.make_request_handle();

        let result1 = handle.make_request(5).await;
        assert_eq!(result1, Some(Ok(10)));

        let reconfig_result = service.reconfigure(config).await;
        assert_eq!(reconfig_result, Some(Ok(())));

        let result2 = handle.make_request(5).await;
        assert_eq!(result2, Some(Ok(10)));

        service.request_immediate_stop().await;
    }

    #[tokio::test]
    async fn test_immediate_stop() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config, 10, TestServiceFactory);
        let handle = service.make_request_handle();

        let result1 = handle.make_request(5).await;
        assert_eq!(result1, Some(Ok(10)));

        service.request_immediate_stop().await;

        let result2 = handle.make_request(5).await;
        assert_eq!(result2, None);
    }

    #[tokio::test]
    async fn test_graceful_stop() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config, 10, TestServiceFactory);
        let handle = service.make_request_handle();

        let result1 = handle.make_request(5).await;
        assert_eq!(result1, Some(Ok(10)));

        service.request_graceful_stop().await;

        let result2 = handle.make_request(5).await;
        assert_eq!(result2, None);
    }

    #[tokio::test]
    async fn test_request_after_immediate_stop() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config, 10, TestServiceFactory);
        let handle = service.make_request_handle();

        service.request_immediate_stop().await;

        let result = handle.make_request(5).await;
        assert_eq!(result, None);
    }

    #[derive(Clone)]
    struct FailingServiceFactory;

    impl Service<TestConfig> for FailingServiceFactory {
        type Response = FailingService;
        type Error = String;
        type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, config: TestConfig) -> Self::Future {
            Box::pin(async move {
                if config.multiplier < 0 {
                    Err("Invalid multiplier".to_string())
                } else {
                    Ok(FailingService { multiplier: config.multiplier })
                }
            })
        }
    }

    struct FailingService {
        multiplier: i32,
    }

    impl Service<i32> for FailingService {
        type Response = i32;
        type Error = String;
        type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: i32) -> Self::Future {
            let result = req * self.multiplier;
            Box::pin(async move { Ok(result) })
        }
    }

    #[tokio::test]
    async fn test_factory_error() {
        let config = TestConfig { multiplier: -1 };
        let service = ReconfigurableService::new(config, 10, FailingServiceFactory);
        let handle = service.make_request_handle();

        let result = handle.make_request(5).await;
        assert_eq!(result, Some(Err("Invalid multiplier".to_string())));

        service.request_immediate_stop().await;
    }

    #[tokio::test]
    async fn test_reconfigure_to_error() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config, 10, FailingServiceFactory);
        let handle = service.make_request_handle();

        let result1 = handle.make_request(5).await;
        assert_eq!(result1, Some(Ok(10)));

        let bad_config = TestConfig { multiplier: -1 };
        let reconfig_result = service.reconfigure(bad_config).await;
        assert_eq!(reconfig_result, Some(Err("Invalid multiplier".to_string())));

        let result2 = handle.make_request(5).await;
        assert_eq!(result2, Some(Err("Invalid multiplier".to_string())));

        service.request_immediate_stop().await;
    }

    #[tokio::test]
    async fn test_buffer_size() {
        let config = TestConfig { multiplier: 1 };
        let service = ReconfigurableService::new(config, 0, TestServiceFactory);
        let handle = service.make_request_handle();

        let result = handle.make_request(42).await;
        assert_eq!(result, Some(Ok(42)));

        service.request_immediate_stop().await;
    }

    #[tokio::test]
    async fn test_multiple_handles() {
        let config = TestConfig { multiplier: 2 };
        let service = ReconfigurableService::new(config, 10, TestServiceFactory);

        let handle1 = service.make_request_handle();
        let handle2 = service.make_request_handle();

        let result1 = handle1.make_request(3).await;
        let result2 = handle2.make_request(7).await;

        assert_eq!(result1, Some(Ok(6)));
        assert_eq!(result2, Some(Ok(14)));

        service.request_immediate_stop().await;
    }
}

