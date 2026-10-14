use axum::{Extension, Json, Router, extract::{Path, Query, State}, http::StatusCode, middleware::from_fn_with_state, response::{IntoResponse, Response}, routing::{delete, get, patch, put}};
use axum_cookie::CookieLayer;
use tonic::Code;
use crate::{clients::{auth::AuthClient, identity::{IdentityClient, identity_grpc}}, handlers::{error::HandlerError, user::dto::{BasicProfileRes, EditProfileReq, ImageTypeQuery, ProfileRes}}, middleware::auth::auth_middleware, util::assets::{AssetPrefix, asset_id_to_url}};

pub mod dto;

#[derive(Clone)]
struct UserState {
    pub identity_client: IdentityClient,
}

#[axum::debug_handler]
async fn get_user_profile(
    Path(user_id): Path<i64>,
    State(state): State<UserState>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;
    
    let req = identity_grpc::FullProfileReq { user_id, with_email: false };
    match identity_client.get_full_profile(req).await {
        Ok(res) => {
            let profile = res.into_inner();
            
            let avatar_url = asset_id_to_url(AssetPrefix::Identity, profile.avatar_id);
            let banner_url = asset_id_to_url(AssetPrefix::Identity, profile.banner_id);

            let res = ProfileRes {
                user_id: profile.user_id,
                username: profile.username,
                email: None,
                avatar_url,
                banner_url,
                first_name: profile.first_name,
                last_name: profile.last_name,
                bio: profile.bio,
                country: profile.country,
                city: profile.city,
            };
            Ok(Json(res).into_response())
        },
        Err(_) => Err(HandlerError::NotFound),
    }
}

#[axum::debug_handler]
async fn get_my_basic_profile(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::UserId { user_id };
    match identity_client.get_basic_profile(req).await {
        Ok(res) => {
            let res = res.into_inner();

            let avatar_url = asset_id_to_url(AssetPrefix::Identity, res.avatar_id);
            Ok(Json(BasicProfileRes {
                user_id: res.user_id,
                username: res.username,
                avatar_url,
            }).into_response())
        },
        Err(_) => Err(HandlerError::NotFound),
    }
}

#[axum::debug_handler]
async fn get_my_full_profile(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::FullProfileReq { user_id, with_email: true };
    match identity_client.get_full_profile(req).await {
        Ok(res) => {
            let res = res.into_inner();

            let avatar_url = asset_id_to_url(AssetPrefix::Identity, res.avatar_id);
            let banner_url = asset_id_to_url(AssetPrefix::Identity, res.banner_id);

            Ok(Json(ProfileRes {
                user_id: res.user_id,
                username: res.username,
                email: res.email,
                avatar_url,
                banner_url,
                first_name: res.first_name,
                last_name: res.last_name,
                bio: res.bio,
                country: res.country,
                city: res.city,
            }).into_response())
        },
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
        },
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
            }).into_response())
        },
        Err(e) => {
            if e.code().eq(&Code::AlreadyExists) {
                Err(HandlerError::Conflict)
            } else {
                Err(HandlerError::InternalError)
            }
        },
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
        },
    }
}

#[axum::debug_handler]
async fn remove_image(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
    Query(query): Query<ImageTypeQuery>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::RemoveImageReq {
        user_id,
    };

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
        },
    }
}

pub fn create_user_router(
    auth_client: AuthClient,
    identity_client: IdentityClient,
) -> Router {
    Router::new()
        .route("/{user_id}", get(get_user_profile))
        .merge(Router::new()
            .route("/me", get(get_my_basic_profile))
            .route("/full-profile", get(get_my_full_profile))
            .route("/update-profile", put(update_my_profile))
            .route("/request-upload-image", patch(request_upload_image))
            .route("/notify-upload-image", patch(notify_upload_image))
            .route("/remove-image", delete(remove_image))
            .layer(from_fn_with_state(auth_client, auth_middleware))
            .layer(CookieLayer::strict())
        )
        .with_state(UserState { identity_client })
}