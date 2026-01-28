use std::{
    error::Error,
    future::{Ready,ready},
    task::{Poll, Context},
};
use tower::{Service, ServiceExt, service_fn};
use crate::{ReloadingInstance,ReloadableService};

type BoxedError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Clone, Default, Debug, PartialEq)]
struct TestConfig {
    value: usize,
}

#[derive(Clone)]
struct TestService {
    inner: usize,
}
impl Service<String> for TestService {
    type Response = String;
    type Error = BoxedError;
    type Future = Ready<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: String) -> Self::Future {
        ready(Ok(format!("{}{}", req, self.inner)))
    }
}

#[derive(Clone)]
struct ConfiguredFactory {
    data: TestConfig,
}
impl Service<()> for ConfiguredFactory {
    type Response = TestService;
    type Error = BoxedError;
    type Future = Ready<Result<Self::Response,Self::Error>>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: ()) -> Self::Future {
        ready(Ok(TestService { inner: self.data.value }))
    }
}

fn build_factory(arg: TestConfig) -> Ready<Result<ConfiguredFactory,BoxedError>> {
    ready(Ok(ConfiguredFactory { data: arg }))
}

#[tokio::test]
async fn basic_validation() {
    let mut instance = ReloadingInstance::new(TestConfig { value: 42 }, service_fn(build_factory))
        .await
        .unwrap();

    let mut x: ReloadableService<ConfiguredFactory,TestService,String> = instance.get_service_handle::<String,TestService>();

    x.ready().await.unwrap();
    let mut service_1 = x.call(()).await.unwrap();
    service_1.ready().await.unwrap();
    let data1 = service_1.call("hello world - ".to_string()).await.unwrap();
    assert_eq!(data1.as_str(),"hello world - 42");

    x.ready().await.unwrap();
    let mut service_2 = x.call(()).await.unwrap();
    service_2.ready().await.unwrap();
    let data2 = service_2.call("foobar - ".to_string()).await.unwrap();
    assert_eq!(data2.as_str(),"foobar - 42");

    instance.reload(TestConfig { value: 67 }).await.unwrap();
    
    // service 1 is now stale
    service_1.ready().await.unwrap();
    let data1 = service_1.call("hello world - ".to_string()).await.unwrap();
    assert_eq!(data1.as_str(),"hello world - 42");

    // service 2 now stale, using the out dated config
    service_2.ready().await.unwrap();
    let data2 = service_2.call("foobar - ".to_string()).await.unwrap();
    assert_eq!(data2.as_str(),"foobar - 42");

    // we can request a new service, getting the up to date config
    x.ready().await.unwrap();
    let mut service_3 = x.call(()).await.unwrap();
    service_3.ready().await.unwrap();
    let data3 = service_3.call("foobar - ".to_string()).await.unwrap();
    assert_eq!(data3.as_str(),"foobar - 67");
}
