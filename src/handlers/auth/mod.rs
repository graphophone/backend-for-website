use axum::{Json, Router, extract::State, response::{IntoResponse, Response}, routing::post};
use validator::Validate;
use crate::{clients::auth::{self, AuthClient}, handlers::error::HandlerError};

mod dto;

async fn login_handler(
    State(mut client): State<AuthClient>,
    Json(req): Json<dto::LoginRequest>,
) -> Result<Response, HandlerError> {
    let login_request = auth::auth::LoginRequest {
        username: req.username,
        password: req.password,
    };

    match client.login(login_request).await {
        Ok(res) => {
            let tokens = res.into_inner();
            let response = dto::Tokens {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            };
            Ok(Json(response).into_response())
        },
        Err(_) => Err(HandlerError::InternalError),
    }
}

#[axum::debug_handler]
async fn sign_up_handler(
    State(mut client): State<AuthClient>,
    Json(req): Json<dto::SignUpRequest>,
) -> Result<Response, HandlerError> {
    if let Err(_) = req.validate() {
        return Err(HandlerError::BadRequest)
    }

    let sign_up_request = auth::auth::SignUpRequest {
        username: req.username,
        email: req.email,
        password: req.password,
        first_name: req.first_name,
        last_name: req.last_name,
    };

    match client.sign_up(sign_up_request).await {
        Ok(res) => {
            let tokens = res.into_inner();
            let response = dto::Tokens {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            };
            Ok(Json(response).into_response())
        },
        Err(_) => Err(HandlerError::InternalError),
    }
}

async fn logout_handler() {
}

async fn refresh_handler() {
}

pub fn create_auth_router(auth_client: AuthClient) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/sign-up", post(sign_up_handler))
        .route("/logout", post(logout_handler))
        .route("/refresh", post(refresh_handler))
        .with_state(auth_client)
}