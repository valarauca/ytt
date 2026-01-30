use std::future::{Ready,ready};
use tower::{
    ServiceExt,
    service_fn,
    util::MapErr,
};
use crate::{
    traits::Err,
    adapters::reconfigurable::{ReconfigurableService},
    adapters::service_tree::{RegisteredServiceTree},
    adapters::service_kind::{ServiceManagement},
    adapters::path_helper::{path_split},
};
use super::config::{ClientConfig,ClientLoader};


pub fn load_default_client<E: Err>(
    tree: RegisteredServiceTree<E>,
    client_config: ClientLoader,
) {
    let path = client_config.path;
    let service = default_loader::<E>(client_config.buffer, client_config.config);
    let manager = ServiceManagement::from(service);
    let path_vec = path_split(&path);
    tree.insert(&path_vec, manager);
}

fn default_loader<E: Err>(
    buffer: usize,
    config: ClientConfig,
) -> ReconfigurableService<ClientConfig,reqwest::Request,reqwest::Response,E> {
    let func = service_fn(factory_impl::<E>);
    ReconfigurableService::new(config, buffer, func)
}

fn factory_impl<E>(config: ClientConfig) -> Ready<Result<ClientErrorFixed<E>,E>>
where
    E: Err,
{
    let mapper: ErrFunc<E> = <E as From<reqwest::Error>>::from;

    let result = config.build()
        .map_err(|e| E::from(e))
        .map(|client: reqwest::Client| -> ClientErrorFixed<E> {
            client.map_err(mapper)
        });
    ready(result)
}

type ErrFunc<E> = fn(reqwest::Error) -> E;
type ClientErrorFixed<E> = MapErr<reqwest::Client,ErrFunc<E>>;
