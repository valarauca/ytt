
use serde::{Serialize,Deserialize};
use openrouter::config::OpenRouterBaseConfig;

#[derive(Serialize,Deserialize,Clone,PartialEq,Debug)]
pub struct OpenRouterConfiguration {
    pub(crate) interior: OpenRouterBaseConfig,
    pub(crate) client: String,
    pub(crate) path: String,
}

