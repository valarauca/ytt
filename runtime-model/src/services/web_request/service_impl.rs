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
};
use super::{
    config::{ClientConfig,ClientLoader},
    service_kind::WebClientService,
};


pub fn load_default_client(
    tree: RegisteredServiceTree,
    client_config: ClientLoader,
) {
    let path = client_config.path;
    let reconfigurable_service = default_loader(client_config.buffer, client_config.config);
    let web_client = WebClientService::new(reconfigurable_service);
    let manager = ServiceManagement::from(web_client);
    // TODO: logging
    let _ = tree.insert(&path, manager);
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

