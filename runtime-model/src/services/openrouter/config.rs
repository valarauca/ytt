
use serde::{Serialize,Deserialize};
use url::Url;
use openrouter::{OpenRouterBaseConfig};

use config_crap::{
    env::{WithEnv},
    boolean::{Boolean},
};
use crate::adapters::{
    path_helper::{GetTreePath,ServiceReqs,IntoServiceConfig,ServiceConfig},
    service_tree::{get_tree},
};

use super::service_impl::{load_client};

#[derive(Serialize,Deserialize,Clone,PartialEq,Debug)]
pub struct OpenRouterConfiguration {
    pub site_url: Url,
    pub api_key: WithEnv<String>,
    pub client_path: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_referer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_completions: Option<Boolean>,
}
impl OpenRouterConfiguration {
    pub(crate) fn chat_completion(&self) -> bool {
        match &self.chat_completions {
            Some(Boolean::True) => true,
            _ => false,
        }
    }
    pub(crate) fn make_base(&self) -> OpenRouterBaseConfig {
        OpenRouterBaseConfig {
            site_url: self.site_url.clone(),
            api_key: self.api_key.to_string(),
            site_title: self.site_title.as_ref().map(|x| x.clone()),
            site_referer: self.site_referer.as_ref().map(|x| x.clone()),
        }
    }
}
impl IntoServiceConfig for OpenRouterConfiguration {
    fn into_service_config(&self) -> ServiceConfig {
        ServiceConfig::new(self.clone())
    }
}
impl ServiceReqs for OpenRouterConfiguration {

    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        self.path.get_tree_path()
    }

    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        Ok(vec![self.client_path.get_tree_path()?])
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
