use std::{
    task::{Context,Poll},
    sync::Arc,
    convert::Infallible,
};
use tower::{Service,ServiceExt,service_fn};
use futures_util::{FutureExt};
use http::{Request as HttpRequest, Response as HttpResponse};
use axum::{
    body::{Body},
    Router,
};

use super::config::{RouterConfig};
use super::config::method::{Method};

use crate::{
    adapters::{
        s3service::{BoxCloneSyncService},
        maybe_async::{make_boxed,make_ready,MaybeFuture},
        service_tree::{get_tree,RegisteredServiceTree},
        service_kind::{ServiceManagement},
        reconfigurable::{ReconfigurableService},
    },
};


pub type ExtHttpRequest = HttpRequest<Body>;
pub type ExtHttpResponse = HttpResponse<Body>;
pub type HttpService = BoxCloneSyncService<ExtHttpRequest,ExtHttpResponse,anyhow::Error>;

pub fn load_client(
    tree: RegisteredServiceTree,
    config: RouterConfig,
) -> anyhow::Result<()> {
    let path = config.path.clone();
    let func = service_fn(factory_impl);
    let service = ReconfigurableService::new(config, 1, func);
    let manager = ServiceManagement::from(service);
    tree.insert(&path, manager)?;
    Ok(())
}

async fn factory_impl(config: RouterConfig) -> anyhow::Result<RouterService> {
    config.build().await
}


/// Interior routing mechancism
pub struct RouterService {
    pub interior: matchit::Router<EndPoint>,
}

#[derive(Default)]
pub struct EndPoint {
    interior: [Option<HttpService>;9]
}
impl std::ops::Index<Method> for EndPoint {
    type Output = Option<HttpService>;
    fn index(&self, idx: Method) -> &Self::Output {
        &self.interior[idx as usize - 1]
    }
}
impl std::ops::IndexMut<Method> for EndPoint {
    fn index_mut(&mut self, idx: Method) -> &mut Self::Output {
        &mut self.interior[idx as usize - 1]
    }
}

impl tower::Service<ExtHttpRequest> for RouterService {
    type Response = ExtHttpResponse;
    type Error = anyhow::Error;
    type Future = MaybeFuture<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ExtHttpRequest) -> Self::Future {
        let method = match Method::from_http(req.method()) {
            Some(m) => m,
            None => return make_405(),
        };
        let m = match self.interior.at_mut(req.uri().path()) {
            Err(_) => {
                let resp = HttpResponse::builder()
                    .status(404)
                    .body(Body::default())
                    .unwrap();
                return make_ready(Ok(resp));
            }
            Ok(m) => m
        };
        match m.value[method].as_mut() {
            None => make_405(),
            Some(s) => {
                let mut service = s.clone();
                make_boxed(async move {
                    service.ready().then(|s| s.unwrap().call(req)).await
                })
            }
        }
    }
}

fn make_405() -> MaybeFuture<Result<ExtHttpResponse,anyhow::Error>> {
    make_ready(Ok(
        HttpResponse::builder()
            .status(405)
            .body(Body::default())
            .unwrap()))
}

