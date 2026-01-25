use std::sync::Arc;

use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder};
use tower::builder::ServiceBuilder;

use crate::config::utils::duration::NiceDuration;



#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub struct Connection {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub overall_connection_deadline: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub layers: Option<Vec<Layer>>,
}
impl Apply for Connection {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {
        let mut b = b;
        b = match &self.overall_connection_deadline {
            &Option::None => b,
            &Option::Some(ref dur) => b.connect_timeout(dur.get_duration()),
        };

        b = match &self.layers {
            &Option::None => b,
            &Option::Some(ref layers) => layers.into_iter().fold(b, |b,layer| layer.add_layer(b)),
        };

        b
    }
}


#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub enum Layer {
    Timeout(NiceDuration),
    ConcurrencyLimit(usize),

    /*
     * Todo: Need to learn more of tower
     *
     */
    //RateLimit { num: usize, per: NiceDuration },
    //RetryLayer { idk }
}
impl Layer {
    fn add_layer(&self, b: ClientBuilder) -> ClientBuilder {
        match self {
            &Self::ConcurrencyLimit(ref num) => {
                b.connector_layer(tower::limit::concurrency::ConcurrencyLimitLayer::new(*num))
            },
            &Self::Timeout(ref dur) => {
                b.connector_layer(tower::timeout::TimeoutLayer::new(dur.get_duration()))
            },
            /*
            &Self::RateLimit { ref num, ref per } => {
                b.connector_layer(tower::util::BoxCloneService::new(tower::limit::rate::RateLimitLayer::new(num.clone() as u64, per.get_duration())))
            },
            */
        }
    }
}
