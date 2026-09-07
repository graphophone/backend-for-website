use std::sync::Arc;

use anyhow::Result;
use axum::{Router};
use tokio::{net::TcpListener, sync::Mutex};

use crate::{clients::{auth::AuthClient}, handlers::auth::create_auth_router};

pub mod config;
mod middleware;
mod clients;
mod handlers;
mod util;

pub async fn run(conf: &config::Config) -> Result<()> {
    let auth_client = AuthClient::build(conf.services.identity.clone()).await?;
    let auth_client = Mutex::new(auth_client);
    let auth_client = Arc::new(auth_client);

    let router = Router::new()
        .nest("/auth", create_auth_router(Arc::clone(&auth_client)));
    
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, router)
        .await
        .map_err(anyhow::Error::from)
}