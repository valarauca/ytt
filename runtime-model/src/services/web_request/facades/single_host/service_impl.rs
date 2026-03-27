use std::{
    pin::Pin,
    task::{Context, Poll},
    future::Future,
    net::{IpAddr},
};
use futures_util::FutureExt;
use tower::{Service,service_fn};
use url::{Url};
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};

use crate::{
    adapters::{
        s3service::{BoxCloneSyncService},
        maybe_async::{make_boxed,make_ready,MaybeFuture},
        service_tree::{get_tree,RegisteredServiceTree},
        service_kind::{ServiceManagement},
        reconfigurable::{ReconfigurableService},
    },
};

use super::{
    config::{SingleHostReverseProxy},
    service_kind::{SingleHostReverseProxyService},
};


pub fn load_client(
    tree: RegisteredServiceTree,
    config: SingleHostReverseProxy,
) -> anyhow::Result<()> {
    let path = config.path.clone();
    let reconfigurable_service = default_loader(config);
    let web_client = SingleHostReverseProxyService::new(reconfigurable_service);
    let manager = ServiceManagement::from(web_client);
    tree.insert(&path, manager)?;
    Ok(())
}

fn default_loader(
    config: SingleHostReverseProxy,
) -> ReconfigurableService<SingleHostReverseProxy,ReqwestRequest,ReqwestResponse> {
    let func = service_fn(factory_impl);
    ReconfigurableService::new(config, 1, func)
}

async fn factory_impl(config: SingleHostReverseProxy) -> anyhow::Result<SingleHostReverseProxyCoreService> {
    let tree = get_tree();
    let forward = tree.get_service(&config.client, ServiceManagement::get_reqwest_web_client).await?;
    Ok(SingleHostReverseProxyCoreService {
        forward,
        url: config.url,
    })
}

/// Handles the schemantics of updating the core URL
pub struct SingleHostReverseProxyCoreService {
    forward: BoxCloneSyncService<ReqwestRequest,ReqwestResponse,anyhow::Error>,
    url: Url,
}
impl Service<ReqwestRequest> for SingleHostReverseProxyCoreService {
    type Response = ReqwestResponse;
    type Error = anyhow::Error;
    type Future = MaybeFuture<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        self.forward.poll_ready(ctx)
    }

    fn call(&mut self, req: ReqwestRequest) -> Self::Future {
        let mut req = req;
        if let Err(e) = update_url(&self.url, req.url_mut()) {
            return make_ready(Err(e));
        }
        self.forward.call(req).right_future()
    }
}

fn update_url(config: &Url, request: &mut Url) -> anyhow::Result<()> {
    if request.cannot_be_a_base() {
        anyhow::bail!("cannot operate on 'cannot-be-a-base' urls such at request '{}'", request);
    }
    let scheme = config.scheme();
    if !scheme.is_empty() && scheme != request.scheme() {
        request.set_scheme(scheme)
            .map_err(|_| anyhow::anyhow!("failed to set scheme '{}' on url '{}'", scheme, &request))?;
    }
    match config.host() {
        None => { },
        Some(url::Host::Ipv4(x)) => {
            request.set_ip_host(IpAddr::from(x))
                .map_err(|_| anyhow::anyhow!("failed to set ipv4 '{}' on url '{}'", x, &request))?;
        }
        Some(url::Host::Ipv6(x)) => {
            request.set_ip_host(IpAddr::from(x))
                .map_err(|_| anyhow::anyhow!("failed to set ipv6 '{}' on url '{}'", x, &request))?;
        }
        Some(url::Host::Domain(s)) => {
            request.set_host(Some(s))
                .map_err(|_| anyhow::anyhow!("failed to set host '{}' on url '{}'", s, &request))?;
        }
    };
    if let Some(port) = config.port() {
        request.set_port(Some(port))
            .map_err(|_| anyhow::anyhow!("failed to set port '{}' on url '{}'", port, &request))?;
    }
    let u = config.username();
    if !u.is_empty() {
        request.set_username(u)
            .map_err(|_| anyhow::anyhow!("failed to set username '{}' on url '{}'", u, &request))?;
    }
    if let Some(pword) = config.password() {
        request.set_password(Some(pword))
            .map_err(|_| anyhow::anyhow!("failed to set password on url '{}'", &request))?;
    }
    Ok(())
}
