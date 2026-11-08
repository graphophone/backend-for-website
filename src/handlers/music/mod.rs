use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
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
        music::{MusicClient, music_grpc},
    },
    handlers::{
        error::HandlerError,
        music::dto::{CategoryRes, FullMusicRes, GetUserMusicQuery, UploadMusicReq},
        user::dto::BasicProfileRes,
    },
    middleware::{
        auth::{auth_middleware, get_user_id_from_cookies},
        owner::owner_middleware,
    },
};

mod dto;

async fn upload_music(
    State(mut music_client): State<MusicClient>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<UploadMusicReq>,
) -> Result<Response, HandlerError> {
    let req = music_grpc::UploadMusicReq {
        title: req.title,
        description: req.description,
        uploader_id: user_id,
        categories_ids: req.categories_ids,
    };
    let res = match music_client.upload_music(req).await {
        Ok(res) => res.into_inner().id,
        Err(_) => return Err(HandlerError::InternalError),
    };
    Ok(Json(res).into_response())
}

#[axum::debug_handler]
async fn request_upload_thumbnail(
    State(mut music_client): State<MusicClient>,
    Path(music_id): Path<i64>,
    Json(req): Json<dto::UplodaReq>,
) -> Result<Response, HandlerError> {
    let req = music_grpc::UploadAssetReq {
        music_id,
        mime_type: req.mime_type,
    };
    match music_client.upload_thumbnail(req).await {
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
    State(mut music_client): State<MusicClient>,
    Path(music_id): Path<i64>,
    Json(req): Json<dto::NotifyUploadAssetReq>,
) -> Result<Response, HandlerError> {
    let req = music_grpc::NotifyUploadAssetReq {
        music_id,
        asset_id: req.asset_id,
    };
    match music_client.notify_upload_thumbnail(req).await {
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
    State(mut music_client): State<MusicClient>,
    Path(music_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = music_grpc::RemoveThumbnailReq { music_id };
    match music_client.remove_thumbnail(req).await {
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

async fn get_full_music(
    State((mut music_client, auth_client, mut identity_client)): State<(
        MusicClient,
        AuthClient,
        IdentityClient,
    )>,
    cookies: CookieManager,
    Path(music_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let user_id = get_user_id_from_cookies(auth_client, cookies).await;
    let user_id = match user_id {
        Ok(v) => Some(v),
        Err(_) => None,
    };

    let req = music_grpc::FullMusicReq { music_id, user_id };
    let music_data = match music_client.get_full_music(req).await {
        Ok(res) => res.into_inner(),
        Err(_) => return Err(HandlerError::InternalError),
    };

    if music_data.upload_status != "uploaded" {
        if let Some(user_id) = user_id {
            if music_data.uploader_id != user_id {
                return Err(HandlerError::Unauthorized);
            }
        } else {
            return Err(HandlerError::Unauthorized);
        }
    }

    let req = identity_grpc::UserId {
        user_id: music_data.uploader_id,
    };
    let uploader_profile = match identity_client.get_basic_profile(req).await {
        Ok(res) => {
            let res = res.into_inner();
            BasicProfileRes {
                user_id: res.user_id,
                username: res.username,
                avatar_url: res.avatar_id,
            }
        }
        Err(_) => return Err(HandlerError::NotFound),
    };

    let res = FullMusicRes {
        id: music_data.id,
        title: music_data.title,
        description: music_data.description,
        thumbnail_url: music_data.thumbnail_id,
        duration_seconds: music_data.duration_seconds,
        play_count: music_data.play_count,
        like_count: music_data.like_count,
        uploader: uploader_profile,
        categories: music_data
            .categories
            .into_iter()
            .map(|c| CategoryRes {
                id: c.id,
                name: c.name,
            })
            .collect(),
        upload_status: music_data.upload_status,
        created_at: music_data.created_at,
        is_liked: music_data.is_liked,
    };

    Ok(Json(res).into_response())
}

#[axum::debug_handler]
async fn request_upload_audio(
    State(mut music_client): State<MusicClient>,
    Path(music_id): Path<i64>,
    Json(req): Json<dto::UplodaReq>,
) -> Result<Response, HandlerError> {
    let req = music_grpc::UploadAssetReq {
        music_id,
        mime_type: req.mime_type,
    };
    match music_client.upload_music_audio(req).await {
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
    State(mut music_client): State<MusicClient>,
    Path(music_id): Path<i64>,
    Json(req): Json<dto::NotifyUploadAssetReq>,
) -> Result<Response, HandlerError> {
    let req = music_grpc::NotifyUploadAssetReq {
        music_id,
        asset_id: req.asset_id,
    };
    match music_client.notify_upload_music_audio(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(HandlerError::InternalError),
    }
}

#[axum::debug_handler]
async fn remove_music(
    State(mut music_client): State<MusicClient>,
    Path(music_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = music_grpc::RemoveMusicReq { music_id };
    match music_client.remove_music(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(HandlerError::InternalError),
    }
}

#[axum::debug_handler]
async fn get_user_music(
    State((mut music_client, auth_client, _)): State<(MusicClient, AuthClient, IdentityClient)>,
    cookies: CookieManager,
    Path(user_id): Path<i64>,
    Query(GetUserMusicQuery { page_number }): Query<GetUserMusicQuery>,
) -> Result<Response, HandlerError> {
    let requester_id = get_user_id_from_cookies(auth_client, cookies).await;
    let requester_id = match requester_id {
        Ok(v) => Some(v),
        Err(_) => None,
    };

    let req = music_grpc::UserMusicReq {
        user_id,
        page_number,
        requester_id,
    };

    let music_page = match music_client.get_user_music(req).await {
        Ok(v) => v.into_inner(),
        Err(_) => return Err(HandlerError::InternalError),
    };

    let is_owner = match requester_id {
        Some(v) => v == user_id,
        None => false,
    };
    let entries = music_page
        .music_entries
        .into_iter()
        .filter_map(|m| {
            if !is_owner && &m.upload_status != "uploaded" {
                return None;
            }
            Some(dto::UserMusicRes {
                id: m.id,
                title: m.title,
                thumbnail_url: m.thumbnail_id,
                duration_seconds: m.duration_seconds,
                play_count: m.play_count,
                like_count: m.like_count,
                upload_status: m.upload_status,
                is_liked: m.is_liked,
            })
        })
        .collect();

    Ok(Json(dto::UserMusicPage {
        music_entries: entries,
        total_pages: music_page.total_pages,
    })
    .into_response())
}

pub fn create_music_router(
    music_client: MusicClient,
    identity_client: IdentityClient,
    auth_client: AuthClient,
) -> Router {
    Router::new()
        .merge(
            Router::new()
                .route("/{music_id}", get(get_full_music))
                .route("/user-music/{user_id}", get(get_user_music))
                .with_state((music_client.clone(), auth_client.clone(), identity_client)),
        )
        .merge(
            Router::new()
                .route("/upload", post(upload_music))
                .merge(
                    Router::new()
                        .route(
                            "/request-upload-image/{music_id}",
                            patch(request_upload_thumbnail),
                        )
                        .route(
                            "/notify-upload-image/{music_id}",
                            patch(notify_upload_thumbnail),
                        )
                        .route(
                            "/request-upload-audio/{music_id}",
                            patch(request_upload_audio),
                        )
                        .route(
                            "/notify-upload-audio/{music_id}",
                            patch(notify_upload_audio),
                        )
                        .route("/{music_id}", delete(remove_music))
                        .route("/remove-image/{music_id}", delete(remove_thumbnail))
                        .layer(from_fn_with_state(music_client.clone(), owner_middleware)),
                )
                .with_state(music_client.clone())
                .layer(from_fn_with_state(auth_client, auth_middleware)),
        )
        .layer(CookieLayer::strict())
}
