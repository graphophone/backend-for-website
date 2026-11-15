use std::collections::HashMap;

use crate::{
    clients::{
        auth::AuthClient,
        follows::{
            FollowsClient,
            follows_grpc::{self, FollowEntry},
        },
        identity::{IdentityClient, identity_grpc},
    },
    handlers::{
        error::HandlerError,
        user::dto::{
            BasicProfileRes, EditProfileReq, FollowsCount, FollowsPageQuery, FollowsPageRes,
            ImageTypeQuery, MyProfileRes, ProfileRes,
        },
    },
    middleware::auth::{auth_middleware, get_user_id_from_cookies},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, put},
};
use axum_cookie::{CookieLayer, CookieManager};
use tonic::Code;

pub mod dto;

#[derive(Clone)]
struct UserState {
    pub auth_client: AuthClient,
    pub identity_client: IdentityClient,
    pub follows_client: FollowsClient,
}

#[axum::debug_handler]
async fn get_user_follows_count(
    Path(user_id): Path<i64>,
    State(state): State<UserState>,
    cookies: CookieManager,
) -> Result<Response, HandlerError> {
    let follower_id = get_user_id_from_cookies(state.auth_client, cookies).await;
    let follower_id = match follower_id {
        Ok(v) => Some(v),
        Err(_) => None,
    };
    let mut follows_client = state.follows_client;

    let req = follows_grpc::GetFollowsCountReq {
        user_id,
        follower_id,
    };

    let res = follows_client
        .get_follows_count(req)
        .await
        .map_err(|_| HandlerError::InternalError)?
        .into_inner();

    Ok(Json(FollowsCount {
        followers_count: res.followers_count,
        followings_count: res.followings_count,
        is_followed: res.is_followed,
    })
    .into_response())
}

#[axum::debug_handler]
async fn get_user_profile(
    Path(user_id): Path<i64>,
    State(state): State<UserState>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::FullProfileReq {
        user_id,
        with_email: false,
    };
    let profile = match identity_client.get_full_profile(req).await {
        Ok(res) => res.into_inner(),
        Err(_) => return Err(HandlerError::NotFound),
    };

    let profile = ProfileRes {
        user_id,
        username: profile.username,
        avatar_url: profile.avatar_id,
        banner_url: profile.banner_id,
        first_name: profile.first_name,
        last_name: profile.last_name,
        bio: profile.bio,
        country: profile.country,
        city: profile.city,
        email: profile.email,
    };

    Ok(Json(profile).into_response())
}

#[axum::debug_handler]
async fn get_my_profile(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::UserId { user_id };
    match identity_client.get_basic_profile(req).await {
        Ok(res) => {
            let res = res.into_inner();

            Ok(Json(MyProfileRes {
                user_id: res.user_id,
                username: res.username,
                avatar_url: res.avatar_id,
            })
            .into_response())
        }
        Err(_) => Err(HandlerError::NotFound),
    }
}

#[axum::debug_handler]
async fn get_my_full_profile(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::FullProfileReq {
        user_id,
        with_email: true,
    };
    match identity_client.get_full_profile(req).await {
        Ok(res) => {
            let res = res.into_inner();

            Ok(Json(ProfileRes {
                user_id: res.user_id,
                username: res.username,
                email: res.email,
                avatar_url: res.avatar_id,
                banner_url: res.banner_id,
                first_name: res.first_name,
                last_name: res.last_name,
                bio: res.bio,
                country: res.country,
                city: res.city,
            })
            .into_response())
        }
        Err(_) => Err(HandlerError::NotFound),
    }
}

#[axum::debug_handler]
async fn update_my_profile(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<EditProfileReq>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::UpdateProfileReq {
        user_id,
        username: req.username,
        email: req.email,
        first_name: req.first_name,
        last_name: req.last_name,
        bio: req.bio,
        country: req.country,
        city: req.city,
    };
    match identity_client.update_profile(req).await {
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
async fn request_upload_image(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<dto::UploadImageReq>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::UploadImageReq {
        user_id,
        mime_type: req.mime_type,
    };
    match identity_client.request_upload_image(req).await {
        Ok(res) => {
            let res = res.into_inner();

            Ok(Json(dto::UploadImageRes {
                upload_url: res.upload_url,
                image_id: res.image_id,
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
async fn notify_upload_image(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
    Query(query): Query<ImageTypeQuery>,
    Json(req): Json<dto::NotifyUploadImageReq>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::NotifyUploadImageReq {
        user_id,
        image_id: req.image_id,
    };
    let res = match query.image_type.as_ref() {
        "avatar" => identity_client.notify_upload_avatar(req).await,
        "banner" => identity_client.notify_upload_banner(req).await,
        _ => return Err(HandlerError::BadRequest("unknown image type".to_string())),
    };
    match res {
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
async fn remove_image(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
    Query(query): Query<ImageTypeQuery>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::RemoveImageReq { user_id };

    let res = match query.image_type.as_ref() {
        "avatar" => identity_client.remove_avatar(req).await,
        "banner" => identity_client.remove_banner(req).await,
        _ => return Err(HandlerError::BadRequest("unknown image type".to_string())),
    };
    match res {
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
async fn follow_user(
    State(state): State<UserState>,
    Extension(follower_id): Extension<i64>,
    Path(followee_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = follows_grpc::ToggleFollowReq {
        followee_id,
        follower_id,
    };
    let mut follows_client = state.follows_client;

    match follows_client.toggle_follow(req).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(HandlerError::InternalError),
    }
}

async fn follows_to_profiles(
    follows: Vec<FollowEntry>,
    mut identity_client: IdentityClient,
) -> anyhow::Result<Vec<BasicProfileRes>> {
    let user_ids = follows.iter().map(|v| v.user_id).collect();
    let req = identity_grpc::UserIds { user_ids };

    let mut profiles: HashMap<_, _> = identity_client
        .get_profiles(req)
        .await?
        .into_inner()
        .profiles
        .into_iter()
        .map(|v| (v.user_id, v))
        .collect();

    let profiles = follows
        .into_iter()
        .filter_map(|f| {
            let profile = match profiles.remove(&f.user_id) {
                Some(v) => v,
                None => return None,
            };
            Some(BasicProfileRes {
                user_id: profile.user_id,
                username: profile.username,
                avatar_url: profile.avatar_id,
                follows_count: f.follows_count.map(|v| dto::FollowsCount {
                    followers_count: v.followers_count,
                    followings_count: v.followings_count,
                    is_followed: v.is_followed,
                }),
            })
        })
        .collect();

    Ok(profiles)
}

#[axum::debug_handler]
async fn get_user_followings(
    State(state): State<UserState>,
    Path(user_id): Path<i64>,
    Query(query): Query<FollowsPageQuery>,
    cookies: CookieManager,
) -> Result<Response, HandlerError> {
    let requestor_id = get_user_id_from_cookies(state.auth_client, cookies).await;
    let requestor_id = match requestor_id {
        Ok(v) => Some(v),
        Err(_) => None,
    };
    let mut follows_client = state.follows_client;

    let req = follows_grpc::GetFollowsReq {
        user_id,
        page_number: query.page_number,
        requestor_id,
    };

    let follows = follows_client
        .get_followings(req)
        .await
        .map_err(|_| HandlerError::InternalError)?
        .into_inner();

    let profiles = follows_to_profiles(follows.follows, state.identity_client)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(Json(FollowsPageRes {
        total_pages: follows.total_pages,
        profiles,
    })
    .into_response())
}

#[axum::debug_handler]
async fn get_user_followers(
    State(state): State<UserState>,
    Path(user_id): Path<i64>,
    Query(query): Query<FollowsPageQuery>,
    cookies: CookieManager,
) -> Result<Response, HandlerError> {
    let requestor_id = get_user_id_from_cookies(state.auth_client, cookies).await;
    let requestor_id = match requestor_id {
        Ok(v) => Some(v),
        Err(_) => None,
    };
    let mut follows_client = state.follows_client;

    let req = follows_grpc::GetFollowsReq {
        user_id,
        page_number: query.page_number,
        requestor_id,
    };

    let follows = follows_client
        .get_followers(req)
        .await
        .map_err(|_| HandlerError::InternalError)?
        .into_inner();

    let profiles = follows_to_profiles(follows.follows, state.identity_client)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(Json(FollowsPageRes {
        total_pages: follows.total_pages,
        profiles,
    })
    .into_response())
}

pub fn create_user_router(
    auth_client: AuthClient,
    identity_client: IdentityClient,
    follows_client: FollowsClient,
) -> Router {
    Router::new()
        .route("/{user_id}", get(get_user_profile))
        .route("/{user_id}/followings", get(get_user_followings))
        .route("/{user_id}/followers", get(get_user_followers))
        .route("/{user_id}/follows-count", get(get_user_follows_count))
        .merge(
            Router::new()
                .route("/me", get(get_my_profile))
                .route("/full-profile", get(get_my_full_profile))
                .route("/update-profile", put(update_my_profile))
                .route("/request-upload-image", patch(request_upload_image))
                .route("/notify-upload-image", patch(notify_upload_image))
                .route("/remove-image", delete(remove_image))
                .route("/{user_id}/follow", patch(follow_user))
                .layer(from_fn_with_state(auth_client.clone(), auth_middleware)),
        )
        .with_state(UserState {
            auth_client,
            identity_client,
            follows_client,
        })
        .layer(CookieLayer::strict())
}
