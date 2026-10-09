use axum::{
    Extension, Json, Router, extract::{Multipart, Path, State}, http::StatusCode, middleware::from_fn_with_state, response::{IntoResponse, Response}, routing::{get, patch, post},
};
use axum_cookie::{CookieLayer, CookieManager};
use tonic::Code;

use crate::{
    clients::{
        auth::AuthClient, tracks::{TracksClient, tracks_grpc},
    }, handlers::{error::HandlerError, tracks::dto::{CategoryRes, FullTrackRes, UploadTrackReq}}, middleware::auth::{auth_middleware, get_user_id_from_cookies}, util::assets::{AssetPrefix, asset_id_to_url, extract_image_data_from_multipart},
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

async fn get_full_track(
    State((mut tracks_client, auth_client)): State<(TracksClient, AuthClient)>,
    cookies: CookieManager,
    Path(track_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::TrackId { id: track_id };

    let track_data = match tracks_client.get_full_track(req).await {
        Ok(res) => res.into_inner(),
        Err(_) => return Err(HandlerError::InternalError),
    };

    if track_data.upload_status != "uploaded" {
        let user_id = get_user_id_from_cookies(auth_client, cookies).await?;
        if track_data.uploader_id != user_id {
            return Err(HandlerError::Unauthorized);
        }
    }

    let thumbnail_url = asset_id_to_url(
        AssetPrefix::Track,
        track_data.thumbnail_id,
    );

    let res = FullTrackRes {
        id: track_data.id,
        title: track_data.title,
        description: track_data.description,
        thumbnail_url,
        duration_seconds: track_data.duration_seconds,
        play_count: track_data.play_count,
        like_count: track_data.like_count,
        uploader_id: track_data.uploader_id,
        categories: track_data.categories
            .into_iter()
            .map(|c| CategoryRes { id: c.id, name: c.name })
            .collect(),
    };

    Ok(Json(res).into_response())
}

pub fn create_tracks_router(tracks_client: TracksClient, auth_client: AuthClient) -> Router {
    Router::new()
        .merge(
            Router::new()
            .route("/{track_id}", get(get_full_track))
            .with_state((tracks_client.clone(), auth_client.clone()))
            .layer(CookieLayer::strict())
        )
        .merge(
            Router::new()
                .route("/upload", post(upload_track))
                .route("/update-thumbnail/{track_id}", patch(update_thumbnail))
                .with_state(tracks_client.clone())
                .layer(from_fn_with_state(auth_client, auth_middleware))
                .layer(CookieLayer::strict())
        )
        .with_state(tracks_client)
}
