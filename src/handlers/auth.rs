use axum::{extract::Json, Router, extract::State, response::{IntoResponse, Response}, routing::post};
use serde::{Deserialize, Serialize};

use crate::{clients::auth::{self, AuthClient}, handlers::error::HandlerError};

#[derive(Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
struct TokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}

async fn login_handler(
    State(mut client): State<AuthClient>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, HandlerError> {
    let login_request = auth::auth::LoginRequest {
        username: req.username,
        password: req.password,
    };

    match client.login(login_request).await {
        Ok(res) => {
            let tokens = res.into_inner();
            let response = TokensResponse {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            };
            Ok(Json(response).into_response())
        },
        Err(_) => Err(HandlerError::InternalError),
    }
}

async fn sign_up_handler(

) {
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