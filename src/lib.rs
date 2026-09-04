use anyhow::Result;
use axum::{Router};
use tokio::net::TcpListener;

pub mod config;
mod handlers;

pub async fn run(conf: &config::Config) -> Result<()> {
    let router = Router::new()
        .nest("/auth", handlers::auth::create_auth_router());
    
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await?;

    axum::serve(listener, router)
        .await
        .map_err(anyhow::Error::from)
}