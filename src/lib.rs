use anyhow::Result;
use axum::{Router};
use tokio::net::TcpListener;

use crate::{clients::{auth::AuthClient, user::UserClient}, handlers::auth::create_auth_router};

pub mod config;
mod clients;
mod handlers;

pub async fn run(conf: &config::Config) -> Result<()> {
    let auth_client = AuthClient::build(conf.services.identity.clone()).await?;
    let user_client = UserClient::build(conf.services.identity.clone()).await?;

    let router = Router::new()
        .nest("/auth", create_auth_router(auth_client));
    
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, router)
        .await
        .map_err(anyhow::Error::from)
}