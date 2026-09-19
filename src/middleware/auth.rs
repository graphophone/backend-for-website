use std::sync::Arc;

use axum::{extract::{Request, State}, middleware::Next, response::Response};
use axum_cookie::CookieManager;
use tokio::sync::Mutex;

use crate::{clients::auth::{self, AuthClient}, handlers::error::HandlerError, util::cookie::extract_access_token};

pub async fn auth_middleware(
    State(state): State<Arc<Mutex<AuthClient>>>,
    cookies: CookieManager,
    mut req: Request,
    next: Next,
) -> Result<Response, HandlerError> {
    let access_token = match extract_access_token(&cookies) {
        Ok(v) => v,
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    let extract_req = auth::auth_grpc::AccessToken {
        access_token,
    };
    let mut auth = state.lock().await;
    let claims = match auth.extract_user_id(extract_req).await {
        Ok(v) => v.into_inner(),
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    req.extensions_mut().insert(claims.user_id);

    Ok(next.run(req).await)
}