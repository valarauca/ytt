use axum::{
    routing::{MethodRouter,Router},
};

use super::config::{RouterConfig};

use runtime_model::adapters::{
    get_tree, ServiceManagement,
};

pub async fn create_router(config: &RouterConfig) -> Result<Router,anyhow::Error> {
    // get global service tree
    let tree = get_tree();

    let mut router = Router::new();
    for (path,endpoint) in config.routes.iter() {
        let tree_path = endpoint.tree_path();
        let service = tree.get_service(&tree_path, ServiceManagement::get_endpoint)
            .await?;
        let filter = endpoint.build_filter();
        let rt = MethodRouter::new()
            .on_service(filter, service);
        router = router.route(path,rt);
    }
    Ok(router)
}

