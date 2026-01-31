use std::collections::HashMap;
use openrouter::{
    OpenRouter,
    Completion,
    completions::{Response as ORResponse},
};
use reqwest::{Request as HttpRequest,Response as HttpResponse};

use super::config::OpenRouterConfiguration;

use crate::{
    adapters::service_tree::{RegisteredServiceTree,get_tree},
    adapters::path_helper::path_split,
    adapters::service_kind::ServiceManagement,
    adapters::maybe_async::{MaybeFuture,make_boxed,make_ready},
};

/*
pub async fn initialize_open_router(
    config: OpenRouterConfiguration,
    tree: RegisteredServiceTree,
) -> Result<(),anyhow::Error> {
    let client_path = config.client;
    let own_path = config.path;
    let config = config.interior;

    let client_path_vec = path_split(&client_path);
    let own_path_vec = path_split(&own_path);

    let client = tree.get_service(&client_path_vec,ServiceManagement::get_web_client).await?;
    let or_client = OpenRouter::<_>::new(config,client);
    // TODO insert model

    let models = or_client.models(&[]).async?;
    let mut models_map = HashMap::new();
    for model in models {
        let endpoints = or_client.models(model.id)?;
    }
}
*/

/*
async fn root_factory_impl(config: OpenRouterConfiguration) -> ReconfigurableService<OpenRouterConfiguration,ORRequest,ORResponse> {
    let client_path = config.client;
    let own_path = config.path;
    let config = config.interior;
    let buffer = config.buffer;

    let client_path_vec = path_split(&client_path);
    let own_path_vec = path_split(&own_path);
    let tree = get_tree();
    let client = tree.get_service(&client_path_vec,ServiceManagement::get_web_client).await?;
}
*/

pub struct OpenRouterService {
    interior: OpenRouter<tower::util::BoxCloneService<HttpRequest,HttpResponse,anyhow::Error>,anyhow::Error>,
    routing_options: Option<Box<dyn FnMut(&mut ORRequest) -> Result<(),anyhow::Error> + Send + 'static>>,
}
impl tower::Service<ORRequest> for OpenRouterService {
    type Response = ORResponse;
    type Error = anyhow::Error;
    type Future = MaybeFuture<Result<Self::Response,Self::Error>>;
    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(),Self::Error>> {
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
        if self.chat {
            let fut = self.interior.chat_completion(req);
        } else {
        }
    }
}
