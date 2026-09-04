use axum::{Router, routing::post};

async fn login_handler() {
}

async fn sign_up_handler() {
}

async fn logout_handler() {
}

pub fn create_auth_router() -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/sign-up", post(sign_up_handler))
        .route("/logout", post(logout_handler))
}