use axum::{Router, routing::post};

use crate::clients::auth::AuthClient;

async fn login_handler() {
}

async fn sign_up_handler() {
}

async fn logout_handler() {
}

pub fn create_auth_router(auth_client: AuthClient) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/sign-up", post(sign_up_handler))
        .route("/logout", post(logout_handler))
        .with_state(auth_client)
}