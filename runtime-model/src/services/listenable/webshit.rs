use std::{
    pin::{Pin},
    future::Future,
    task::{Context,Poll},
    convert::Infallible,
};

use tower::{Service};

use crate::{
    services::listenable::kinds::{ExtHttpRequest,ExtHttpResponse},
    adapters::s3service::{BoxCloneSyncService},
};

pub type InputKind = BoxCloneSyncService<ExtHttpRequest,ExtHttpResponse,anyhow::Error>;

#[derive(Clone)]
pub struct Webshit {
    interior: InputKind
}
impl From<InputKind> for Webshit {
    fn from(interior: InputKind) -> Webshit {
        Self { interior }
    }
}
impl Service<ExtHttpRequest> for Webshit {
    type Response = ExtHttpResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>> + Send + 'static>>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: ExtHttpRequest) -> Self::Future {
        let fut = self.interior.call(req);
        Box::pin(async move {
            http_shit(fut.await)
        })
    }
}

fn http_shit(r: Result<ExtHttpResponse,anyhow::Error>) -> Result<ExtHttpResponse,std::convert::Infallible> {
    use axum::response::IntoResponse;
    match r {
        Ok(x) => Ok(x),
        Err(_) => {
            Ok(http::status::StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
