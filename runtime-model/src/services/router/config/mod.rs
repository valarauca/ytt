pub mod method;
mod route_methods;

use std::{
	collections::{HashSet,HashMap,BTreeMap},
};
use futures_util::{
    future::{FutureExt,TryFutureExt},
    stream::{TryStreamExt,futures_unordered::FuturesUnordered},
};
use serde::{Deserialize};
use self::route_methods::MethodConfig;

use crate::adapters::{
    ServiceManagement,
    path_helper::{GetTreePath,ServiceReqs,IntoServiceConfig,ServiceConfig},
    service_tree::{get_tree},
};

use super::service_impl::{EndPoint,ExtHttpRequest,ExtHttpResponse,HttpService,RouterService,load_client};


#[derive(Deserialize,Clone,Debug,PartialEq,Eq)]
pub struct RouterConfig {
    pub path: String,
    pub routes: BTreeMap<String,MethodConfig>,
}
impl IntoServiceConfig for RouterConfig {
    fn into_service_config(&self) -> ServiceConfig {
        ServiceConfig::new(self.clone())
    }
}
impl ServiceReqs for RouterConfig {

    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        self.path.get_tree_path()
    }

    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        self.routes.keys()
            .map(|k| k.get_tree_path())
            .collect::<anyhow::Result<Vec<Vec<&'a str>>>>()
    }

    fn insert_to_tree(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=anyhow::Result<()>> + Send + 'static>> {
        let s = self.clone();
        Box::pin(async move {
            let path = s.path.clone();
            let tree = get_tree();
            if tree.contains_path(&path)? {
                tree.reload(&path, Box::new(s)).await?;
            } else {
                load_client(tree, s)?;
            }
            Ok(())
        })
    }
}

impl RouterConfig {
    pub async fn build(&self) -> anyhow::Result<RouterService> {
		let tree = get_tree();
		let services = self
            .routes
            .values()
            .flat_map(|mc| mc.info.keys().map(|s| s.as_str()))
            .collect::<HashSet<&str>>() // implicit dedup
            .into_iter()
            .map(|name| tree.get_service(name, ServiceManagement::get_endpoint).map_ok(move |x| (name,x)))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<HashMap<&str,HttpService>>()
            .await?;

        let iter = self.routes.iter()
            .map(|(route,config)| -> (&str,EndPoint) {
                let end_point = config.info.iter()
                    .flat_map(|(service_id, method)| method.iter().map(|method| (method,service_id.as_str())))
                    .fold(EndPoint::default(), |mut point,(method,service_id)| {
                        point[*method] = Some(services.get(service_id).unwrap().clone());
                        point
                    });
                (route,end_point)
            });
        let mut interior = matchit::Router::new();
        for (route, endpoint) in iter {
            interior.insert(route, endpoint)?;
        }
        Ok(RouterService { interior })
    }
}
