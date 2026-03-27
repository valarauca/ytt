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

use super::config::OpenRouterConfiguration;

pub fn load_client(
    tree: RegisteredServiceTree,
    config: OpenRouterConfiguration,
) -> anyhow::Result<()> {
    let path = config.path.clone();
    let func = service_fn(factory_impl);
    let service = ReconfigurableService::new(config, 1, func);
    let manager = ServiceManagement::from(service);
    tree.insert(&path, manager)?;
    Ok(())
}

async fn factory_impl(config: OpenRouterConfiguration) -> anyhow::Result<OpenRouterService> {
    let chat_completion = config.chat_completion();
    let base = config.make_base();
    let tree = get_tree();
    let forward = tree.get_service(&config.client_path, ServiceManagement::get_reqwest_web_client).await?;
    let client = OpenRouter::new(base, forward);
    Ok(OpenRouterService {
        interior: client,
        chat_completion,
    })
}


#[derive(Clone)]
pub struct OpenRouterService {
    interior: OpenRouter<BoxCloneSyncService<ReqwestRequest,ReqwestResponse,anyhow::Error>>,
    chat_completion: bool,
}
impl tower::Service<ORRequest> for OpenRouterService {
    type Response = ORResponse;
    type Error = anyhow::Error;
    type Future = MaybeFuture<Result<Self::Response,Self::Error>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        self.interior.service.poll_ready(ctx)
    }

    fn call(&mut self, req: ORRequest) -> Self::Future {
        let req = req;
        if self.chat_completion {
            self.interior.call(ChatCompletion(req))
        } else {
            self.interior.call(Completion(req))
        }
    }
}
