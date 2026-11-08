use axum::{
    Extension, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::patch,
};
use axum_cookie::CookieLayer;

use crate::{
    clients::{
        auth::AuthClient,
        likes::{LikesClient, likes_grpc::LikeMusicReq},
    },
    handlers::error::HandlerError,
    middleware::auth::auth_middleware,
};

#[axum::debug_handler]
async fn like_music(
    State(mut likes_client): State<LikesClient>,
    Extension(user_id): Extension<i64>,
    Path(music_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = LikeMusicReq { music_id, user_id };

    match likes_client.like_music(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(HandlerError::InternalError),
    }
}

pub fn create_likes_router(likes_client: LikesClient, auth_client: AuthClient) -> Router {
    Router::new()
        .route("/{music_id}/like", patch(like_music))
        .with_state(likes_client.clone())
        .layer(from_fn_with_state(auth_client, auth_middleware))
        .layer(CookieLayer::strict())
}
