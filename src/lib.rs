use std::sync::Arc;

use anyhow::Result;
use axum::{Router};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use crate::{clients::{auth::AuthClient, identity::identity_grpc::identity_client::IdentityClient}, handlers::auth::create_auth_router};

pub mod config;
mod middleware;
mod clients;
mod handlers;
mod util;

pub async fn run(conf: config::Config) -> Result<()> {
    tracing_subscriber::fmt::init();

    let auth_client = AuthClient::build(
        conf.services.auth.clone(),
    ).await?;
    let auth_client = Arc::new(Mutex::new(auth_client));
    
    let identity_client = IdentityClient::build(
        conf.services.identity.clone(),
    ).await?;
    let identity_client = Arc::new(Mutex::new(identity_client));
    
    let auth_conf = Arc::new(conf.auth);

    let router = Router::new()
        .nest("/auth", create_auth_router(
            Arc::clone(&identity_client),
            Arc::clone(&auth_client),
            Arc::clone(&auth_conf),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive());
    
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, router)
        .await
        .map_err(anyhow::Error::from)
}
