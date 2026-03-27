use std::{
    collections::{HashMap},
    task::{Context,Poll},
    sync::Arc,
};
use futures_util::FutureExt;
use tower::{Service,service_fn};
use openrouter::{
    OpenRouter,
    Completion, ChatCompletion,
    completions::{Response as ORResponse, Request as ORRequest},
};
use reqwest::{Request as ReqwestRequest, Response as ReqwestResponse};
use url::{Url};

use crate::{
    adapters::{
        s3service::{BoxCloneSyncService},
        maybe_async::{make_boxed,make_ready,MaybeFuture},
        service_tree::{get_tree,RegisteredServiceTree},
        service_kind::{ServiceManagement},
        reconfigurable::{ReconfigurableService},
    },
};

use super::config::{DefaultsConfig,OpenRouterSettingsConfig};


pub fn load_client(
    tree: RegisteredServiceTree,
    config: OpenRouterSettingsConfig,
) -> anyhow::Result<()> {
    let path = config.path.clone();
    let func = service_fn(factory_impl);
    let service = ReconfigurableService::new(config, 1, func);
    let manager = ServiceManagement::from(service);
    tree.insert(&path, manager)?;
    Ok(())
}

async fn factory_impl(config: OpenRouterSettingsConfig) -> anyhow::Result<OpenRouterDefaultValuesService> {
    let tree = get_tree();
    let forward = tree.get_service(&config.open_router_path, ServiceManagement::get_openrouter).await?;
    Ok(OpenRouterDefaultValuesService {
        interior: forward,
        config: config.defaults,
    })
}

pub struct OpenRouterDefaultValuesService {
    interior: BoxCloneSyncService<ORRequest,ORResponse,anyhow::Error>,
    config: DefaultsConfig,
}
impl Service<ORRequest> for OpenRouterDefaultValuesService {
    type Response = ORResponse;
    type Error = anyhow::Error;
    type Future = MaybeFuture<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        self.interior.poll_ready(ctx)
    }

    fn call(&mut self, req: ORRequest) -> Self::Future {
        let mut req = req;
        self.config.update_request(&mut req);
        self.interior.call(req).right_future()
    }
}
