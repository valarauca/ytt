use std::{
    pin::Pin,
    task::{Context, Poll},
    future::Future,
};
use tower::Service;
use http::{Request as HttpRequest, Response as HttpResponse};
use axum::body::{Body as AxumBody};
use hyper::body::{Incoming as HyperBody};
use http_body_util::{BodyExt, combinators::BoxBody, BodyDataStream, StreamBody};
use http_body::Frame;
use futures_util::{Stream,StreamExt};
use bytes::Bytes;
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};

use crate::{
    adapters::reconfigurable::{RequestHandle, ReconfigurableService},
    traits::{BoxedConfig, type_error},
    services::web_request::service_kind::{StreamingBody,PinnedFuture,StdError,PinnedStreamBody},
};
use super::config::SingleHostReverseProxy;


pub struct SingleHostReverseProxyService {
    service: ReconfigurableService<SingleHostReverseProxy,ReqwestRequest,ReqwestResponse>,
}
impl SingleHostReverseProxyService {

    pub fn new(service: ReconfigurableService<SingleHostReverseProxy,ReqwestRequest,ReqwestResponse>) -> Self {
        Self { service }
    }

    pub async fn reload(&self, config: BoxedConfig) -> anyhow::Result<()> {
        let config = config.downcast::<SingleHostReverseProxy>()
            .map_err(|_| type_error::<SingleHostReverseProxy>())?;
        self.service.reconfigure(*config).await
    }

    pub fn make_reqwest_service(&self) -> ReverseProxyReqwestServiceHandle {
        ReverseProxyReqwestServiceHandle {
            handle: self.service.make_request_handle(),
        }
    }

    pub fn make_axum_service(&self) -> ReverseProxyAxumServiceHandle {
        ReverseProxyAxumServiceHandle {
            handle: self.service.make_request_handle(),
        }
    }

    pub fn make_hyper_service(&self) -> ReverseProxyHyperServiceHandle {
        ReverseProxyHyperServiceHandle {
            handle: self.service.make_request_handle(),
        }
    }
}

pub struct ReverseProxyReqwestServiceHandle {
    handle: RequestHandle<SingleHostReverseProxy, ReqwestRequest, ReqwestResponse>,
}

impl Clone for ReverseProxyReqwestServiceHandle {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl Service<ReqwestRequest> for ReverseProxyReqwestServiceHandle {
    type Response = ReqwestResponse;
    type Error = anyhow::Error;
    type Future = PinnedFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.handle.poll_ready(cx)
    }

    fn call(&mut self, req: ReqwestRequest) -> Self::Future {
        self.handle.call(req)
    }
}

pub struct ReverseProxyAxumServiceHandle {
    handle: RequestHandle<SingleHostReverseProxy, ReqwestRequest, ReqwestResponse>,
}

impl Clone for ReverseProxyAxumServiceHandle {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl Service<HttpRequest<AxumBody>> for ReverseProxyAxumServiceHandle {
    type Response = HttpResponse<AxumBody>;
    type Error = anyhow::Error;
    type Future = PinnedFuture<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest<AxumBody>) -> Self::Future {
        let mut handle = self.handle.clone();
        Box::pin(async move {

            let s = req.uri().to_string();
            let url = s.parse::<reqwest::Url>()?;
            let (parts, body) = req.into_parts();
            let mut request = ReqwestRequest::new(parts.method, url);
            *request.headers_mut() = parts.headers;
            *request.body_mut() = Some(reqwest::Body::wrap_stream(body.into_data_stream()));
            *request.version_mut() = parts.version;

            let resp = handle.call(request).await?;

            let mut b = http::response::Builder::new()
                .status(resp.status())
                .version(resp.version());
            b.headers_mut().unwrap().clone_from(resp.headers());
            let stream: Pin<Box<dyn Stream<Item=Result<Bytes,StdError>> + Send + 'static>> = Box::pin(
                resp.bytes_stream()
                    .map(|x| -> Result<Bytes,Box<dyn std::error::Error + Send + Sync + 'static>> {
                        match x {
                            Ok(x) => Ok(x),
                            Err(e) => Err(Box::new(e))
                        }
                    })
            );
            Ok(b.body(AxumBody::from_stream(stream))?)
        })
    }
}

pub struct ReverseProxyHyperServiceHandle {
    handle: RequestHandle<SingleHostReverseProxy, ReqwestRequest, ReqwestResponse>,
}

impl Clone for ReverseProxyHyperServiceHandle {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}


impl Service<HttpRequest<HyperBody>> for ReverseProxyHyperServiceHandle {
    type Response = HttpResponse<StreamingBody>;
    type Error = StdError;
    type Future = PinnedFuture<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest<HyperBody>) -> Self::Future {
        let mut handle = self.handle.clone();
        Box::pin(async move {

            // setup the request
            let s = req.uri().to_string();
            let url = s.parse::<reqwest::Url>()?;
            let (parts, body) = req.into_parts();
            let mut request = ReqwestRequest::new(parts.method, url);
            *request.headers_mut() = parts.headers;
            *request.body_mut() = Some(reqwest::Body::wrap(body));
            *request.version_mut() = parts.version;

            let resp = handle.call(request).await?;

            // reformat the response
            let mut b = http::response::Builder::new()
                .status(resp.status())
                .version(resp.version());
            b.headers_mut().unwrap().clone_from(resp.headers());
            let stream: PinnedStreamBody = Box::pin(
                resp.bytes_stream()
                    .map(|x| -> Result<Frame<Bytes>,Box<dyn std::error::Error + Send + Sync>> {
                        match x {
                            Ok(x) => Ok(Frame::data(x)),
                            Err(e) => Err(Box::new(e))
                        }
                    })
            );
            let stream: StreamingBody = StreamBody::new(stream);
            Ok(b.body(stream)?)
        })
    }
}

