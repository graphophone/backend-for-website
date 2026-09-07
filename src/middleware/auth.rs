use std::sync::Arc;

use axum::{extract::{Request, State}, middleware::Next, response::Response};
use axum_cookie::CookieManager;
use tokio::sync::Mutex;

use crate::{clients::auth::{self, AuthClient}, handlers::error::HandlerError, util::cookie::extract_tokens};

pub async fn auth_middleware(
    State(client): State<Arc<Mutex<AuthClient>>>,
    cookies: CookieManager,
    mut req: Request,
    next: Next,
) -> Result<Response, HandlerError> {
    let (access_token, refresh_token) = match extract_tokens(&cookies) {
        Ok(v) => v,
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    let mut c = client.lock().await;
    let claims = c.extract_claims(auth::auth::Tokens { access_token, refresh_token }).await;
    let claims = match claims {
        Ok(v) => v.into_inner(),
        Err(_) => return Err(HandlerError::Unauthorized),
    };

    req.extensions_mut().insert(claims.user_id);

    Ok(next.run(req).await)
}