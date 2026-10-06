use axum::{
    Extension, Json, Router, extract::State, middleware::from_fn_with_state, response::{IntoResponse, Response}, routing::{patch, post},
};
use axum_cookie::CookieLayer;

use crate::{
    clients::{
        auth::AuthClient,
        tracks::{TracksClient, tracks_grpc},
    },
    handlers::{error::HandlerError, tracks::dto::UploadTrackReq},
    middleware::auth::auth_middleware,
};

mod dto;

async fn upload_track(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<UploadTrackReq>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::UploadTrackReq {
        title: req.title,
        description: req.description,
        uploader_id: user_id,
        categories_ids: req.categories_ids,
    };
    let res = match tracks_client.upload_track(req).await {
        Ok(res) => res.into_inner().id,
        Err(_) => return Err(HandlerError::InternalError),
    };
    Ok(Json(res).into_response())
}

async fn update_thumbnail(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<UploadTrackReq>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::UploadTrackReq {
        title: req.title,
        description: req.description,
        uploader_id: user_id,
        categories_ids: req.categories_ids,
    };
    let res = match tracks_client.upload_track(req).await {
        Ok(res) => res.into_inner().id,
        Err(_) => return Err(HandlerError::InternalError),
    };
    Ok(Json(res).into_response())
}

pub fn create_tracks_router(tracks_client: TracksClient, auth_client: AuthClient) -> Router {
    Router::new()
        .route("/upload", post(upload_track))
        .route("/update-thumbnail", patch(update_thumbnail))
        .with_state(tracks_client)
        .layer(from_fn_with_state(auth_client, auth_middleware))
        .layer(CookieLayer::strict())
}
