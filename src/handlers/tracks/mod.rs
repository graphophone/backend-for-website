use axum::{
    Extension, Json, Router, extract::{Multipart, Path, State}, http::StatusCode, middleware::from_fn_with_state, response::{IntoResponse, Response}, routing::{patch, post},
};
use axum_cookie::CookieLayer;
use tonic::Code;

use crate::{
    clients::{
        auth::AuthClient,
        tracks::{TracksClient, tracks_grpc},
    }, handlers::{error::HandlerError, tracks::dto::UploadTrackReq}, middleware::auth::auth_middleware, util::assets::extract_image_data_from_multipart,
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

#[axum::debug_handler]
async fn update_thumbnail(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Path(track_id): Path<i64>,
    image_multipart: Option<Multipart>,
) -> Result<Response, HandlerError> {
    let image_data = extract_image_data_from_multipart(image_multipart)
        .await?;

    let image_payload = match image_data {
        Some(data) => {
            let image = tracks_grpc::Image {
                image: data.image_bytes,
                mime_type: data.mime_type,
            };
            Some(image)
        },
        None => None,
    };

    let req = tracks_grpc::UpdateTrackThumbnailReq {
        track_id,
        thumbnail: image_payload,
        uploader_id: user_id,
    };
    match tracks_client.update_track_thumbnail(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(e) => {
            if e.code().eq(&Code::AlreadyExists) {
                Err(HandlerError::Conflict)
            } else {
                Err(HandlerError::InternalError)
            }
        },
    }
}

pub fn create_tracks_router(tracks_client: TracksClient, auth_client: AuthClient) -> Router {
    Router::new()
        .route("/upload", post(upload_track))
        .route("/update-thumbnail/{track_id}", patch(update_thumbnail))
        .with_state(tracks_client)
        .layer(from_fn_with_state(auth_client, auth_middleware))
        .layer(CookieLayer::strict())
}
