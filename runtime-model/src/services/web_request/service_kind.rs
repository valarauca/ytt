use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use http::{Request as HttpRequest, Response as HttpResponse};
use axum::body::Body as AxumBody;
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};
use http_body_util::BodyExt;

use crate::{
    adapters::reconfigurable::{RequestHandle, ReconfigurableService},
    traits::{BoxedConfig, type_error},
};
use super::config::ClientConfig;

pub struct WebClientService {
    service: ReconfigurableService<ClientConfig, ReqwestRequest, ReqwestResponse>,
}

impl WebClientService {
    pub fn new(service: ReconfigurableService<ClientConfig, ReqwestRequest, ReqwestResponse>) -> Self {
        Self { service }
    }

    pub async fn reload(&self, config: BoxedConfig) -> anyhow::Result<()> {
        let client_config = config.downcast::<ClientConfig>()
            .map_err(|_| type_error::<ClientConfig>())?;
        self.service.reconfigure(*client_config).await
    }

    pub fn make_reqwest_service(&self) -> ReqwestServiceHandle {
        ReqwestServiceHandle {
            handle: self.service.make_request_handle(),
        }
    }

    pub fn make_axum_service(&self) -> AxumServiceHandle {
        AxumServiceHandle {
            handle: self.service.make_request_handle(),
        }
    }
}

pub struct ReqwestServiceHandle {
    handle: RequestHandle<ClientConfig, ReqwestRequest, ReqwestResponse>,
}

impl Clone for ReqwestServiceHandle {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl Service<ReqwestRequest> for ReqwestServiceHandle {
    type Response = ReqwestResponse;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.handle.poll_ready(cx)
    }

    fn call(&mut self, req: ReqwestRequest) -> Self::Future {
        self.handle.call(req)
    }
}

pub struct AxumServiceHandle {
    handle: RequestHandle<ClientConfig, ReqwestRequest, ReqwestResponse>,
}

impl Clone for AxumServiceHandle {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl Service<HttpRequest<AxumBody>> for AxumServiceHandle {
    type Response = HttpResponse<AxumBody>;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest<AxumBody>) -> Self::Future {
        let mut handle = self.handle.clone();
        Box::pin(async move {
            let reqwest_request = convert_axum_to_reqwest(req).await?;
            let reqwest_response = handle.call(reqwest_request).await?;
            let axum_response = convert_reqwest_to_axum(reqwest_response).await?;
            Ok(axum_response)
        })
    }
}

pub async fn convert_axum_to_reqwest(
    req: HttpRequest<AxumBody>
) -> anyhow::Result<ReqwestRequest> {
    let (parts, body) = req.into_parts();

    let body_bytes = body
        .collect()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to collect body: {}", e))?
        .to_bytes();

    let url = parts.uri.to_string();

    let mut reqwest_builder = reqwest::Client::new()
        .request(parts.method, &url);

    for (name, value) in parts.headers.iter() {
        reqwest_builder = reqwest_builder.header(name, value);
    }

    if !body_bytes.is_empty() {
        reqwest_builder = reqwest_builder.body(body_bytes);
    }

    reqwest_builder.build()
        .map_err(|e| anyhow::anyhow!("Failed to build reqwest request: {}", e))
}

pub async fn convert_reqwest_to_axum(
    resp: ReqwestResponse
) -> anyhow::Result<HttpResponse<AxumBody>> {
    let status = resp.status();
    let headers = resp.headers().clone();
    let body_bytes = resp.bytes().await
        .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

    let mut response = HttpResponse::builder()
        .status(status);

    for (name, value) in headers.iter() {
        response = response.header(name, value);
    }

    response.body(AxumBody::from(body_bytes))
        .map_err(|e| anyhow::anyhow!("Failed to build axum response: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Method, StatusCode};

    #[tokio::test]
    async fn test_convert_axum_to_reqwest_simple_get() {
        let req = HttpRequest::builder()
            .method(Method::GET)
            .uri("http://example.com/test")
            .body(AxumBody::empty())
            .unwrap();

        let result = convert_axum_to_reqwest(req).await;
        assert!(result.is_ok());

        let reqwest_req = result.unwrap();
        assert_eq!(reqwest_req.method(), Method::GET);
        assert!(reqwest_req.url().as_str().contains("/test"));
    }

    #[tokio::test]
    async fn test_convert_axum_to_reqwest_with_headers() {
        let req = HttpRequest::builder()
            .method(Method::POST)
            .uri("http://example.com/api/endpoint")
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer token123")
            .body(AxumBody::empty())
            .unwrap();

        let result = convert_axum_to_reqwest(req).await;
        assert!(result.is_ok());

        let reqwest_req = result.unwrap();
        assert_eq!(reqwest_req.method(), Method::POST);
        assert_eq!(
            reqwest_req.headers().get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(
            reqwest_req.headers().get("Authorization").unwrap(),
            "Bearer token123"
        );
    }

    #[tokio::test]
    async fn test_convert_axum_to_reqwest_with_body() {
        let body_content = "test body content";
        let req = HttpRequest::builder()
            .method(Method::POST)
            .uri("http://example.com/submit")
            .body(AxumBody::from(body_content))
            .unwrap();

        let result = convert_axum_to_reqwest(req).await;
        assert!(result.is_ok());

        let reqwest_req = result.unwrap();
        assert_eq!(reqwest_req.method(), Method::POST);
    }

    #[tokio::test]
    async fn test_convert_reqwest_to_axum_basic() {
        use reqwest::Client;

        let client = Client::new();
        let reqwest_resp = client
            .get("https://httpbin.org/status/200")
            .send()
            .await;

        if let Ok(resp) = reqwest_resp {
            let result = convert_reqwest_to_axum(resp).await;
            assert!(result.is_ok());

            let axum_resp = result.unwrap();
            assert_eq!(axum_resp.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_convert_reqwest_to_axum_with_headers() {
        use reqwest::Client;

        let client = Client::new();
        let reqwest_resp = client
            .get("https://httpbin.org/response-headers?Custom-Header=TestValue")
            .send()
            .await;

        if let Ok(resp) = reqwest_resp {
            let result = convert_reqwest_to_axum(resp).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_convert_reqwest_to_axum_with_body() {
        use reqwest::Client;
        use http_body_util::BodyExt;

        let client = Client::new();
        let reqwest_resp = client
            .get("https://httpbin.org/json")
            .send()
            .await;

        if let Ok(resp) = reqwest_resp {
            let result = convert_reqwest_to_axum(resp).await;
            assert!(result.is_ok());

            let axum_resp = result.unwrap();
            let body = axum_resp.into_body();
            let bytes = body.collect().await.unwrap().to_bytes();
            assert!(!bytes.is_empty());
        }
    }

    #[tokio::test]
    async fn test_round_trip_conversion() {
        let original_body = b"Hello, World!";
        let axum_req = HttpRequest::builder()
            .method(Method::POST)
            .uri("http://example.com/echo")
            .header("Content-Type", "text/plain")
            .body(AxumBody::from(&original_body[..]))
            .unwrap();

        let result = convert_axum_to_reqwest(axum_req).await;
        assert!(result.is_ok());
    }
}
