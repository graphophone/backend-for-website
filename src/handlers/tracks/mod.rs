use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use axum_cookie::{CookieLayer, CookieManager};
use tonic::Code;

use crate::{
    clients::{
        auth::AuthClient,
        identity::{IdentityClient, identity_grpc},
        tracks::{TracksClient, tracks_grpc},
    },
    handlers::{
        error::HandlerError,
        tracks::dto::{CategoryRes, FullTrackRes, UploadTrackReq},
        user::dto::BasicProfileRes,
    },
    middleware::auth::{auth_middleware, get_user_id_from_cookies},
    util::assets::{AssetPrefix, asset_id_to_url},
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
async fn request_upload_thumbnail(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Path(track_id): Path<i64>,
    Json(req): Json<dto::UplodaReq>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::UploadAssetReq {
        track_id,
        mime_type: req.mime_type,
        uploader_id: user_id,
    };
    match tracks_client.upload_thumbnail(req).await {
        Ok(res) => {
            let res = res.into_inner();

            Ok(Json(dto::UploadAssetRes {
                upload_url: res.upload_url,
                asset_id: res.asset_id,
            })
            .into_response())
        }
        Err(e) => {
            if e.code().eq(&Code::AlreadyExists) {
                Err(HandlerError::Conflict)
            } else {
                Err(HandlerError::InternalError)
            }
        }
    }
}

#[axum::debug_handler]
async fn notify_upload_thumbnail(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Path(track_id): Path<i64>,
    Json(req): Json<dto::NotifyUploadAssetReq>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::NotifyUploadAssetReq {
        track_id,
        asset_id: req.asset_id,
        uploader_id: user_id,
    };
    match tracks_client.notify_upload_thumbnail(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(e) => {
            if e.code().eq(&Code::AlreadyExists) {
                Err(HandlerError::Conflict)
            } else {
                Err(HandlerError::InternalError)
            }
        }
    }
}

#[axum::debug_handler]
async fn remove_thumbnail(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Path(track_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::RemoveThumbnailReq {
        track_id,
        uploader_id: user_id,
    };
    match tracks_client.remove_thumbnail(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(e) => {
            if e.code().eq(&Code::AlreadyExists) {
                Err(HandlerError::Conflict)
            } else {
                Err(HandlerError::InternalError)
            }
        }
    }
}

async fn get_full_track(
    State((mut tracks_client, auth_client, mut identity_client)): State<(
        TracksClient,
        AuthClient,
        IdentityClient,
    )>,
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

    let thumbnail_url = asset_id_to_url(AssetPrefix::Track, track_data.thumbnail_id);

    let req = identity_grpc::UserId {
        user_id: track_data.uploader_id,
    };
    let uploader_profile = match identity_client.get_basic_profile(req).await {
        Ok(res) => {
            let res = res.into_inner();
            let avatar_url = asset_id_to_url(AssetPrefix::Identity, res.avatar_id);
            BasicProfileRes {
                user_id: res.user_id,
                username: res.username,
                avatar_url,
            }
        }
        Err(_) => return Err(HandlerError::NotFound),
    };

    let res = FullTrackRes {
        id: track_data.id,
        title: track_data.title,
        description: track_data.description,
        thumbnail_url,
        duration_seconds: track_data.duration_seconds,
        play_count: track_data.play_count,
        like_count: track_data.like_count,
        uploader: uploader_profile,
        categories: track_data
            .categories
            .into_iter()
            .map(|c| CategoryRes {
                id: c.id,
                name: c.name,
            })
            .collect(),
        upload_status: track_data.upload_status,
        created_at: track_data.created_at,
    };

    Ok(Json(res).into_response())
}

#[axum::debug_handler]
async fn request_upload_audio(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Path(track_id): Path<i64>,
    Json(req): Json<dto::UplodaReq>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::UploadAssetReq {
        track_id,
        mime_type: req.mime_type,
        uploader_id: user_id,
    };
    match tracks_client.upload_thumbnail(req).await {
        Ok(res) => {
            let res = res.into_inner();

            Ok(Json(dto::UploadAssetRes {
                upload_url: res.upload_url,
                asset_id: res.asset_id,
            })
            .into_response())
        }
        Err(e) => {
            if e.code().eq(&Code::AlreadyExists) {
                Err(HandlerError::Conflict)
            } else {
                Err(HandlerError::InternalError)
            }
        }
    }
}

#[axum::debug_handler]
async fn notify_upload_audio(
    State(mut tracks_client): State<TracksClient>,
    Extension(user_id): Extension<i64>,
    Path(track_id): Path<i64>,
    Json(req): Json<dto::NotifyUploadAssetReq>,
) -> Result<Response, HandlerError> {
    let req = tracks_grpc::NotifyUploadAssetReq {
        track_id,
        asset_id: req.asset_id,
        uploader_id: user_id,
    };
    match tracks_client.notify_upload_track_audio(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(HandlerError::InternalError),
    }
}

pub fn create_tracks_router(
    tracks_client: TracksClient,
    identity_client: IdentityClient,
    auth_client: AuthClient,
) -> Router {
    Router::new()
        .merge(
            Router::new()
                .route("/{track_id}", get(get_full_track))
                .with_state((tracks_client.clone(), auth_client.clone(), identity_client))
                .layer(CookieLayer::strict()),
        )
        .merge(
            Router::new()
                .route("/upload", post(upload_track))
                .route(
                    "/request-upload-image/{track_id}",
                    patch(request_upload_thumbnail),
                )
                .route(
                    "/notify-upload-image/{track_id}",
                    patch(notify_upload_thumbnail),
                )
                .route(
                    "/request-upload-audio/{track_id}",
                    patch(request_upload_audio),
                )
                .route(
                    "/notify-upload-audio/{track_id}",
                    patch(notify_upload_audio),
                )
                .route("/remove-image/{track_id}", delete(remove_thumbnail))
                .with_state(tracks_client.clone())
                .layer(from_fn_with_state(auth_client, auth_middleware))
                .layer(CookieLayer::strict()),
        )
}
