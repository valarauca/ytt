use std::{
    task::{Poll, Context},
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
};

use tower::{Service, ServiceExt};
use futures_util::future::BoxFuture;

use crate::instance::ReloadingInstance;
use crate::reloadable::ReloadableService;

#[derive(Clone)]
struct MockRequest(String);

#[derive(Clone)]
struct MockResponse(String);

#[derive(Clone)]
struct SimpleService {
    id: Arc<AtomicUsize>,
}

impl SimpleService {
    fn new(id: usize) -> Self {
        Self {
            id: Arc::new(AtomicUsize::new(id)),
        }
    }

    fn get_id(&self) -> usize {
        self.id.load(Ordering::SeqCst)
    }
}

impl Service<MockRequest> for SimpleService {
    type Response = MockResponse;
    type Error = SimpleServiceError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: MockRequest) -> Self::Future {
        let id = self.get_id();
        Box::pin(async move {
            Ok(MockResponse(format!("{}-{}", id, req.0)))
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct SimpleServiceError;

impl std::fmt::Display for SimpleServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleServiceError")
    }
}

impl std::error::Error for SimpleServiceError {}

impl crate::reloadable::Err for SimpleServiceError {
    fn stopped() -> Self {
        SimpleServiceError
    }
}

#[derive(Clone, Default, Debug, mirror_mirror::Reflect)]
struct TestConfig {
    value: usize,
}

struct SimpleFactory;

impl Service<TestConfig> for SimpleFactory {
    type Response = SimpleService;
    type Error = SimpleFactoryError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, config: TestConfig) -> Self::Future {
        Box::pin(async move {
            Ok(SimpleService::new(config.value))
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct SimpleFactoryError;

impl std::fmt::Display for SimpleFactoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleFactoryError")
    }
}

impl std::error::Error for SimpleFactoryError {}

#[tokio::test]
async fn test_reloading_instance_creation() {
    let config = TestConfig { value: 42 };
    let factory = SimpleFactory;

    let instance = ReloadingInstance::new(config, factory)
        .await
        .expect("failed to create instance");

    let mut service: ReloadableService<SimpleServiceError, MockRequest, SimpleService> =
        instance.service();

    service.ready().await.expect("service not ready");
    let req = MockRequest("test".to_string());
    let result = service.call(req).await.expect("call failed");

    assert_eq!(result.0, "42-test");
}

#[tokio::test]
async fn test_multiple_service_subscribers() {
    let config = TestConfig { value: 42 };
    let factory = SimpleFactory;

    let instance = ReloadingInstance::new(config, factory)
        .await
        .expect("failed to create instance");

    let mut service1: ReloadableService<SimpleServiceError, MockRequest, SimpleService> =
        instance.service();
    let mut service2: ReloadableService<SimpleServiceError, MockRequest, SimpleService> =
        instance.service();

    service1.ready().await.expect("service1 not ready");
    service2.ready().await.expect("service2 not ready");

    let req1 = MockRequest("first".to_string());
    let req2 = MockRequest("second".to_string());

    let result1 = service1.call(req1).await.expect("call 1 failed");
    let result2 = service2.call(req2).await.expect("call 2 failed");

    assert_eq!(result1.0, "42-first");
    assert_eq!(result2.0, "42-second");
}

#[tokio::test]
async fn test_channel_provides_initial_service() {
    let config = TestConfig { value: 42 };
    let factory = SimpleFactory;

    let instance = ReloadingInstance::new(config, factory)
        .await
        .expect("failed to create instance");

    let channel = instance.channel();
    let mut service: ReloadableService<SimpleServiceError, MockRequest, SimpleService> =
        ReloadableService::new(channel);

    let mut ctx = Context::from_waker(&std::task::Waker::noop());

    match service.poll_ready(&mut ctx) {
        Poll::Ready(Ok(())) => {
            let req = MockRequest("ready".to_string());
            let result = service.call(req).await.expect("call failed");
            assert_eq!(result.0, "42-ready");
        }
        other => panic!("expected Ready(Ok(())), got {:?}", other),
    }
}

#[tokio::test]
async fn test_service_ready_state() {
    let config = TestConfig { value: 42 };
    let factory = SimpleFactory;

    let instance = ReloadingInstance::new(config, factory)
        .await
        .expect("failed to create instance");

    let mut service: ReloadableService<SimpleServiceError, MockRequest, SimpleService> =
        instance.service();

    let mut ctx = Context::from_waker(&std::task::Waker::noop());

    let poll_result = service.poll_ready(&mut ctx);
    match poll_result {
        Poll::Ready(Ok(())) => {}
        Poll::Pending => {
            panic!("service should be ready");
        }
        Poll::Ready(Err(_)) => {
            panic!("service should not error on ready");
        }
    }
}

#[tokio::test]
async fn test_sequential_requests() {
    let config = TestConfig { value: 42 };
    let factory = SimpleFactory;

    let instance = ReloadingInstance::new(config, factory)
        .await
        .expect("failed to create instance");

    let mut service: ReloadableService<SimpleServiceError, MockRequest, SimpleService> =
        instance.service();

    service.ready().await.expect("service not ready");
    for i in 0..5 {
        let req = MockRequest(format!("request_{}", i));
        let result = service.call(req).await.expect("call failed");
        assert_eq!(result.0, format!("42-request_{}", i));
    }
}

#[tokio::test]
async fn test_service_reload() {
    let initial_config = TestConfig { value: 42 };
    let mut instance = ReloadingInstance::new(initial_config, SimpleFactory)
        .await
        .expect("failed to create instance");

    let mut service: ReloadableService<SimpleServiceError, MockRequest, SimpleService> =
        instance.service();

    service.ready().await.expect("service not ready");
    let req = MockRequest("before_reload".to_string());
    let result = service.call(req).await.expect("call failed");
    assert_eq!(result.0, "42-before_reload");

    instance.ready().await.expect("instance not ready");
    let new_config = TestConfig { value: 44 };
    let _ = instance.call(new_config).await.expect("reload failed");

    service.ready().await.expect("service not ready after reload");
    let req = MockRequest("after_reload".to_string());
    let result = service.call(req).await.expect("call failed");
    assert_eq!(result.0, "44-after_reload");
}
