use serde::{Deserialize,Serialize};
use url::{Url};

use crate::adapters::{
    path_helper::{GetTreePath,ServiceReqs,IntoServiceConfig,ServiceConfig},
    service_tree::{get_tree},
};

use super::service_impl::{load_client};


#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug)]
pub struct SingleHostReverseProxyConfig {
    pub client: String,
    pub path: String,
    pub url: Url,
}
impl IntoServiceConfig for SingleHostReverseProxyConfig {
    fn into_service_config(&self) -> ServiceConfig {
        ServiceConfig::new(self.clone())
    }
}
impl ServiceReqs for SingleHostReverseProxyConfig {

    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        self.path.get_tree_path()
    }

    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        Ok(vec![self.client.get_tree_path()?])
    }
    fn insert_to_tree(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=anyhow::Result<()>> + Send + 'static>> {
        let s = self.clone();
        Box::pin(async move {
            let path = s.path.clone();
            let tree = get_tree();
            if tree.contains_path(&path)? {
                tree.reload(&path, Box::new(s.path.clone())).await?;
            } else {
                load_client(tree,s)?;
            }
            Ok(())
        })
    }
}

