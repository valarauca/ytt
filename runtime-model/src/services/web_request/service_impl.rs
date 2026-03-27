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
};


pub fn load_client(
    tree: RegisteredServiceTree,
    client_config: ClientLoader,
) -> anyhow::Result<()> {
    let path = client_config.path;
    let r = default_loader(client_config.config);
    let manager = ServiceManagement::from(r);
    tree.insert(&path, manager)?;
    Ok(())
}

fn default_loader(
    config: ClientConfig,
) -> ReconfigurableService<ClientConfig,reqwest::Request,reqwest::Response> {
    let func = service_fn(factory_impl);
    ReconfigurableService::new(config, 1, func)
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

