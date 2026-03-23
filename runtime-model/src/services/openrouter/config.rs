
use serde::{Serialize,Deserialize};
use openrouter::config::OpenRouterBaseConfig;
use crate::adapters::{
    get_tree,
    path_split,
    MaybeFuture,
};

#[derive(Serialize,Deserialize,Clone,PartialEq,Debug)]
pub struct OpenRouterConfiguration {
    pub(crate) interior: OpenRouterBaseConfig,
    pub(crate) buffer: usize,
    pub(crate) client: String,
    pub(crate) path: String,
    pub(crate) chat_completions: bool,
}
/*
impl crate::services::traits::ServiceConfig for OpenRouterConfiguration {
    fn can_initialize(&self) -> MaybeFuture {
        let path = path_split(&self.client);
        let tree = tree;  
        make_boxed(async move {
            tree.get_service(&client_path,ServiceManagement::get_web_client).await.is_ok()
        })
    }
    fn initialize(&self) -> MaybeFuture<Result<(),anyhow::Error>> {
    }
}
*/
