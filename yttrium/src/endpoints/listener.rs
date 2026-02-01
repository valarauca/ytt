use std::{
    future::Future,
};
use tokio::net::{
    TcpListener,
};
use super::{
    config::{RouterConfig},
    builder::{create_router},
};

pub async fn listener<F>(
    config: &RouterConfig,
    stop: F,
) -> anyhow::Result<()> 
where
    F: Future<Output=()> + Send + 'static,
{
    let routes = create_router(config).await?;
    let listener = TcpListener::bind(&config.socket).await?;
    axum::serve(listener, routes)
        .with_graceful_shutdown(stop)
        .await?;
    Ok(())
}
