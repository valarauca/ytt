use std::{
    task::{Poll,Context},
    pin::Pin,
};
use tower::{Service};

use crate::{
    adapters::service_tree::{RegisteredServiceTree},
    traits::{RegisteredService,Err,BoxedConfig,ServiceObj},
};


pub struct ServiceTree<E: Err> {
    inner: RegisteredServiceTree<E>,
}


pub struct ServiceReloadRequest {
    pub path: Vec<String>,
    pub data: BoxedConfig,
}


impl<E: Err> Service<ServiceReloadRequest> for ServiceTree<E> {
    type Response = ();
    type Error = E;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>> + 'static + Send>>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: ServiceReloadRequest) -> Self::Future {
        let tree = self.inner.clone();
        Box::pin(async move {
            let request: Vec<&str> = req.path.iter().map(|s| s.as_str()).collect();
            let data = req.data;
            tree.reload(&request, data).await
        })
    }
}


pub struct ServiceGetRequest<E: Err,Req,Res> {
    pub path: Vec<String>,
    pub data: fn(&dyn RegisteredService<E>) -> Result<ServiceObj<Req,Res,E>,E>,
}
pub struct ServiceGetResponse<E: Err, Req, Res> {
    pub data: ServiceObj<Req,Res,E>,
}


impl<E: Err,Req,Res> Service<ServiceGetRequest<E,Req,Res>> for ServiceTree<E>
where
    Req: 'static + Send,
    Res: 'static + Send,
{
    type Response = ServiceGetResponse<E, Req, Res>;
    type Error = E;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>> + 'static + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: ServiceGetRequest<E,Req,Res>) -> Self::Future {
        let tree = self.inner.clone();
        Box::pin(async move {
            let request: Vec<&str> = req.path.iter().map(|s| s.as_str()).collect();
            Ok(ServiceGetResponse {
                data: tree.get_service(&request, req.data).await?,
            })
        })
    }
}


pub struct ServiceGetOrParentRequest<E: Err,Req,Res> {
    pub path: Vec<String>,
    pub data: fn(&dyn RegisteredService<E>) -> Result<ServiceObj<Req,Res,E>,E>,
}
pub struct ServiceGetOrParentResponse<E: Err, Req, Res> {
    pub data: ServiceObj<Req,Res,E>,
}
impl<E: Err,Req,Res> Service<ServiceGetOrParentRequest<E,Req,Res>> for ServiceTree<E>
where
    Req: 'static + Send,
    Res: 'static + Send,
{
    type Response = ServiceGetOrParentResponse<E, Req, Res>;
    type Error = E;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>> + 'static + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: ServiceGetOrParentRequest<E,Req,Res>) -> Self::Future {
        let tree = self.inner.clone();
        Box::pin(async move {
            let request: Vec<&str> = req.path.iter().map(|s| s.as_str()).collect();
            Ok(ServiceGetOrParentResponse {
                data: tree.get_or_parent_service(&request, req.data).await?,
            })
        })
    }
}

/*
pub struct ContainsPathRequest {
    pub data: Vec<String>,
}
impl<E: Err,Req,Res> Service<ContainsPathRequest> for ServiceTree<E>
where
    Req: 'static + Send,
    Res: 'static + Send,
{
    type Response = bool;
    type Error = E;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>> + 'static + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: ServiceGetOrParentRequest<E,Req,Res>) -> Self::Future {
        let tree = self.inner.clone();
        Box::pin(async move {
            let request: Vec<&str> = req.path.iter().map(|s| s.as_str()).collect();
            Ok(ServiceGetOrParentResponse {
                data: tree.get_or_parent_service(&request, req.data).await?,
            })
        })
    }
}
*/
