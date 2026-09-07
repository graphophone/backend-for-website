use std::sync::Arc;
use axum::{Json, Router, extract::State, http::StatusCode, response::{IntoResponse, Response}, routing::{delete, post}};
use axum_cookie::CookieManager;
use tokio::sync::Mutex;
use validator::Validate;
use crate::{clients::auth::{self, AuthClient, auth::Tokens}, handlers::error::HandlerError, util};

mod dto;

#[axum::debug_handler]
async fn login_handler(
    State(client): State<Arc<Mutex<AuthClient>>>,
    Json(req): Json<dto::LoginRequest>,
) -> Result<Response, HandlerError> {
    let login_request = auth::auth::LoginRequest {
        username: req.username,
        password: req.password,
    };

    let mut c = client.lock().await;
    match c.login(login_request).await {
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
    State(client): State<Arc<Mutex<AuthClient>>>,
    Json(req): Json<dto::SignUpRequest>,
) -> Result<Response, HandlerError> {
    if let Err(e) = req.validate() {
        let desc = e.field_errors()
            .into_iter()
            .next()
            .unwrap().1
            .into_iter()
            .next()
            .unwrap()
            .to_string();
        return Err(HandlerError::BadRequest(desc));
    }

    let sign_up_request = auth::auth::SignUpRequest {
        username: req.username,
        email: req.email,
        password: req.password,
        first_name: req.first_name,
        last_name: req.last_name,
    };

    let mut c = client.lock().await;
    match c.sign_up(sign_up_request).await {
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

async fn logout_handler(
    State(client): State<Arc<Mutex<AuthClient>>>,
    cookies: CookieManager,
) -> Result<Response, HandlerError> {
    let (access_token, refresh_token) = match util::cookie::extract_tokens(cookies) {
        Ok(v) => v,
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    let tokens = Tokens {
        access_token,
        refresh_token,
    };
    let mut c = client.lock().await;
    match c.logout(tokens).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(HandlerError::InternalError),
    }
}

async fn refresh_handler(
    State(client): State<Arc<Mutex<AuthClient>>>,
    cookies: CookieManager,
) -> Result<Response, HandlerError> {
    let (access_token, refresh_token) = match util::cookie::extract_tokens(cookies) {
        Ok(v) => v,
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    let tokens = Tokens {
        access_token,
        refresh_token,
    };
    let mut c = client.lock().await;
    match c.refresh_tokens(tokens).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(HandlerError::InternalError),
    }
}

pub fn create_auth_router(auth_client: Arc<Mutex<AuthClient>>) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/sign-up", post(sign_up_handler))
        .route("/logout", delete(logout_handler))
        .route("/refresh", post(refresh_handler))
        .with_state(auth_client)
}