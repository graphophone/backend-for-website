use std::sync::Arc;

use crate::{
    clients::{
        auth::AuthClient, categories::CategoriesClient,
        identity::identity_grpc::identity_client::IdentityClient, music::MusicClient,
    },
    handlers::{
        auth::create_auth_router, categories::create_categories_router, music::create_music_router,
        user::create_user_router,
    },
};
use anyhow::Result;
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

mod clients;
pub mod config;
mod handlers;
mod middleware;
mod util;

pub async fn run(conf: config::Config) -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let auth_client = AuthClient::build(conf.services.auth.clone()).await?;

    let identity_client = IdentityClient::build(conf.services.identity.clone()).await?;

    let categories_client = CategoriesClient::build(conf.services.music_catalog.clone()).await?;

    let music_client = MusicClient::build(conf.services.music_catalog.clone()).await?;

    let auth_conf = Arc::new(conf.auth);

    let router = Router::new()
        .nest(
            "/auth",
            create_auth_router(
                identity_client.clone(),
                auth_client.clone(),
                Arc::clone(&auth_conf),
            ),
        )
        .nest(
            "/user",
            create_user_router(auth_client.clone(), identity_client.clone()),
        )
        .nest("/categories", create_categories_router(categories_client))
        .nest(
            "/music",
            create_music_router(music_client, identity_client.clone(), auth_client.clone()),
        )
        .route(
            "/health",
            get(async || -> Response { StatusCode::OK.into_response() }),
        )
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    println!("backend for frontend started");
    axum::serve(listener, router)
        .await
        .map_err(anyhow::Error::from)
}
