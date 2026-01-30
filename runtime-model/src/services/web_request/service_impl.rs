use std::future::{Ready,ready};
use tower::{
    ServiceExt,
    service_fn,
    util::MapErr,
};
use crate::{
    adapters::reconfigurable::ReconfigurableService,
    adapters::service_tree::RegisteredServiceTree,
    adapters::service_kind::ServiceManagement,
    adapters::path_helper::path_split,
};
use super::config::{ClientConfig,ClientLoader};


pub fn load_default_client(
    tree: RegisteredServiceTree,
    client_config: ClientLoader,
) {
    let path = client_config.path;
    let service = default_loader(client_config.buffer, client_config.config);
    let manager = ServiceManagement::from(service);
    let path_vec = path_split(&path);
    tree.insert(&path_vec, manager);
}

fn default_loader(
    buffer: usize,
    config: ClientConfig,
) -> ReconfigurableService<ClientConfig,reqwest::Request,reqwest::Response> {
    let func = service_fn(factory_impl);
    ReconfigurableService::new(config, buffer, func)
}

fn factory_impl(config: ClientConfig) -> Ready<Result<ClientErrorFixed,anyhow::Error>>
{
    let mapper: ErrFunc = |e: reqwest::Error| anyhow::Error::from(e);

    let result = config.build()
        .map_err(|e| anyhow::Error::from(e))
        .map(|client: reqwest::Client| -> ClientErrorFixed {
            client.map_err(mapper)
        });
    ready(result)
}

type ErrFunc = fn(reqwest::Error) -> anyhow::Error;
type ClientErrorFixed = MapErr<reqwest::Client,ErrFunc>;
