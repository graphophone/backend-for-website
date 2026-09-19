use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use crate::{clients::{auth::AuthClient, identity::identity_grpc::identity_client::IdentityClient}, handlers::{auth::create_auth_router, user::create_user_router}};

pub mod config;
mod middleware;
mod clients;
mod handlers;
mod util;

pub async fn run(conf: config::Config) -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let auth_client = AuthClient::build(
        conf.services.auth.clone(),
    ).await?;
    
    let identity_client = IdentityClient::build(
        conf.services.identity.clone(),
    ).await?;
    
    let auth_conf = Arc::new(conf.auth);

    let router = Router::new()
        .nest("/auth", create_auth_router(
            identity_client.clone(),
            auth_client.clone(),
            Arc::clone(&auth_conf),
        ))
        .nest("/user", create_user_router(
            identity_client.clone(),
        ))
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http());
    
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, router)
        .await
        .map_err(anyhow::Error::from)
}
