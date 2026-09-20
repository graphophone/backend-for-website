use std::sync::Arc;
use axum::{Json, Router, extract::State, http::StatusCode, response::{IntoResponse, Response}, routing::{delete, patch, post}};
use axum_cookie::{CookieLayer, CookieManager};
use tonic::Code;
use validator::Validate;
use crate::{clients::{auth::{AuthClient, auth_grpc}, identity::{IdentityClient, identity_grpc}}, config::AuthConfig, handlers::error::HandlerError, util::cookie::{add_token_cookies, extract_refresh_token, remove_token_cookies}};

mod dto;

#[derive(Clone)]
struct AuthState {
    pub identity_client: IdentityClient,
    pub auth_client: AuthClient,
    pub auth_conf: Arc<AuthConfig>,
}

#[axum::debug_handler]
async fn login_handler(
    cookies: CookieManager,
    State(state): State<AuthState>,
    Json(req): Json<dto::LoginReq>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;
    let mut auth_client = state.auth_client;
    
    let verify_req = identity_grpc::VerifyPasswordReq {
        username: req.username,
        password: req.password,
    };

    let res = identity_client.verify_password(verify_req).await;
    let user_id = match res {
        Ok(user_id) => user_id.into_inner().user_id,
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    let login_req = auth_grpc::UserId { user_id };
    let res = auth_client.login(login_req).await;
    match res {
        Ok(res) => {
            let tokens = res.into_inner();
            add_token_cookies(
                &cookies,
                (tokens.access_token, tokens.refresh_token),
                &state.auth_conf,
            );
            Ok(StatusCode::OK.into_response())
        },
        Err(_) => Err(HandlerError::InternalError),
    }
}

#[axum::debug_handler]
async fn sign_up_handler(
    cookies: CookieManager,
    State(state): State<AuthState>,
    Json(req): Json<dto::SignUpReq>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;
    let mut auth_client = state.auth_client;

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

    let create_req = identity_grpc::CreateUserReq {
        username: req.username,
        email: req.email,
        password: req.password,
        first_name: req.first_name,
        last_name: req.last_name,
    };

    let user_id = match identity_client.create_user(create_req).await {
        Ok(v) => v.into_inner().user_id,
        Err(e) => {
            if e.code().eq(&Code::AlreadyExists) {
                return Err(HandlerError::Conflict);
            }
            return Err(HandlerError::InternalError);
        },
    };

    let login_req = auth_grpc::UserId { user_id };
    match auth_client.login(login_req).await {
        Ok(res) => {
            let tokens = res.into_inner();
            add_token_cookies(
                &cookies,
                (tokens.access_token, tokens.refresh_token),
                &state.auth_conf,
            );
            Ok(StatusCode::OK.into_response())
        },
        Err(_) => Ok(StatusCode::CREATED.into_response()),
    }
}

async fn logout_handler(
    cookies: CookieManager,
    State(state): State<AuthState>,
) -> Result<Response, HandlerError> {
    let mut auth_client = state.auth_client;

    let refresh_token = match extract_refresh_token(&cookies) {
        Ok(v) => v,
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    let logout_req = auth_grpc::RefreshToken {
        refresh_token,
    };
    match auth_client.logout(logout_req).await {
        Ok(_) => {
            remove_token_cookies(&cookies);
            Ok(StatusCode::OK.into_response())
        },
        Err(_) => Err(HandlerError::InternalError),
    }
}

async fn refresh_handler(
    cookies: CookieManager,
    State(state): State<AuthState>,
) -> Result<Response, HandlerError> {
    let mut auth_client = state.auth_client;

    let refresh_token = match extract_refresh_token(&cookies) {
        Ok(v) => v,
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    let refresh_req = auth_grpc::RefreshToken {
        refresh_token,
    };
    match auth_client.refresh_tokens(refresh_req).await {
        Ok(res) => {
            let tokens = res.into_inner();
            add_token_cookies(
                &cookies,
                (tokens.access_token, tokens.refresh_token),
                &state.auth_conf,
            );
            Ok(StatusCode::OK.into_response())
        },
        Err(_) => Err(HandlerError::Unauthorized),
    }
}

pub fn create_auth_router(
    identity_client: IdentityClient,
    auth_client: AuthClient,
    auth_conf: Arc<AuthConfig>,
) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/sign-up", post(sign_up_handler))
        .route("/logout", delete(logout_handler))
        .route("/refresh", patch(refresh_handler))
        .with_state(AuthState { identity_client, auth_client, auth_conf })
        .layer(CookieLayer::strict())
}
