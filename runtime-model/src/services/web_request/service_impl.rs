use std::future::{Ready,ready};
use tower::{
    ServiceExt,
    service_fn,
    util::MapErr,
};
use crate::{
    traits::Err,
    adapters::reconfigurable::{ReconfigurableService},
};
use super::config::{ClientConfig};



pub fn default_loader<E: Err>(
    buffer: usize,
    config: ClientConfig,
) -> ReconfigurableService<ClientConfig,reqwest::Request,reqwest::Response,E> {
    let func = service_fn(factory_impl::<E>);
    ReconfigurableService::new(config, buffer, func)
}

fn factory_impl<E>(config: ClientConfig) -> Ready<Result<ClientErrorFixed<E>,E>>
where
    E: Err + Sized,
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

