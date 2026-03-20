use std::{
    collections::{HashMap},
    task::{Context,Poll},
    sync::Arc,
};
use tower::{service_fn};
use openrouter::{
    OpenRouter,
    Completion, ChatCompletion,
    completions::{Response as ORResponse, Request as ORRequest},
};
use reqwest::{Request as HttpRequest,Response as HttpResponse};
use futures_util::future::{Either,FutureExt,TryFuture,TryFutureExt};

use super::config::OpenRouterConfiguration;

use crate::{
    adapters::{
        reconfigurable::ReconfigurableService,
        RegisteredServiceTree,get_tree,
        MaybeFuture,make_boxed,make_ready,
        ServiceManagement,
        BoxCloneSyncService,
        path_split, 
    },
};

fn default_loader(config: OpenRouterConfiguration) -> ReconfigurableService<OpenRouterConfiguration,ORRequest,ORResponse> {
    let func = service_fn(root_factory_impl);
    let buffer = config.buffer;
    ReconfigurableService::new(config, buffer, func)
}

async fn root_factory_impl(config: OpenRouterConfiguration) -> Result<OpenRouterService,anyhow::Error> {
    use openrouter::config::OpenRouterBaseConfig;

    let base_config: OpenRouterBaseConfig = config.interior;
    let chat: bool = config.chat_completions;

    let tree = get_tree();

    let client: BoxCloneSyncService<HttpRequest,HttpResponse,anyhow::Error>  = {
        let client_path = path_split(&config.client);
        tree.get_service(&client_path,ServiceManagement::get_web_client).await?
    };
    Ok(OpenRouterService {
        interior: OpenRouter::new(base_config,client),
        routing_options: None,
        chat_completion: chat,
    })
}



#[derive(Clone)]
pub struct OpenRouterService {
    interior: OpenRouter<BoxCloneSyncService<HttpRequest,HttpResponse,anyhow::Error>>,
    routing_options: Option<Arc<dyn Fn(&mut ORRequest) -> Result<(),anyhow::Error> + Send + Sync + 'static>>,
    chat_completion: bool,
}
impl tower::Service<ORRequest> for OpenRouterService {
    type Response = ORResponse;
    type Error = anyhow::Error;
    type Future = MaybeFuture<Result<Self::Response,Self::Error>>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
        // This will tunnel through to `reqwest::Client` "eventually" so this hsould be fine
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: ORRequest) -> Self::Future {
        let mut req = req;
        if let Some(lambda) = &mut self.routing_options {
            match (lambda)(&mut req) {
                Ok(()) => { },
                Err(e) => return make_ready(Err(e)),
            };
        }
        if self.chat_completion {
            self.interior.call(ChatCompletion(req))
        } else {
            self.interior.call(Completion(req))
        }
    }
}
