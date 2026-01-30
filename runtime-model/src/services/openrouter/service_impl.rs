use std::collections::HashMap;
use openrouter::{OpenRouter};

use super::config::{OpenRouterConfiguration};

use crate::{
    traits::Err,
    adapters::service_tree::RegisteredServiceTree,
    adapters::path_helper::{path_split},
    adapters::service_kind::{ServiceManagement},
};

/*
pub async fn initialize_open_router<E: Err>(
    config: OpenRouterConfiguration,
    tree: RegisteredServiceTree<E>,
) -> Result<(),E> {
    let client_path = config.client;
    let own_path = config.path;
    let config = config.interior;

    let client_path_vec = path_split(&client_path);
    let own_path_vec = path_split(&own_path);

    let client = tree.get_service(&client_path_vec,ServiceManagement::<E>::get_web_client).await?;
    let or_client = OpenRouter::<_,E>::new(config,client);
    // TODO insert model

    let models = or_client.models(&[]).async?;
    let mut models_map = HashMap::new();
    for model in models {
        let endpoints = or_client.models(model.id)?;
    }
}
*/


fn factory_impl<E>(config:(
