use axum::{Extension, Json, Router, extract::{Multipart, Path, State}, http::StatusCode, middleware::from_fn_with_state, response::{IntoResponse, Response}, routing::{get, put}};
use axum_cookie::CookieLayer;
use tonic::Code;
use crate::{clients::{auth::AuthClient, identity::{IdentityClient, identity_grpc}}, handlers::{error::HandlerError, user::dto::{BasicProfileRes, EditProfileReq, ProfileRes}}, middleware::auth::auth_middleware, util::assets::asset_key_to_url};

mod dto;

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
            
            let avatar_url = asset_key_to_url(profile.avatar_id);
            let banner_url = asset_key_to_url(profile.banner_id);

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

            let avatar_url = asset_key_to_url(res.avatar_id);
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

            let avatar_url = asset_key_to_url(res.avatar_id);
            let banner_url = asset_key_to_url(res.banner_id);

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
async fn edit_my_profile(
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
async fn edit_my_avatar(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
    image_multipart: Option<Multipart>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let image_data = match image_multipart {
        Some(mut image_multipart) => {
            let field = image_multipart.next_field()
                .await
                .map_err(|_| HandlerError::BadRequest("Image has to be sent".to_string()))?;            
            let bytes = match field {
                Some(field) => {
                    let mime_type = match field.content_type() {
                        Some(v) => {
                            let split = v.split("/");
                            let mime_type = split.skip(1).next();
                            match mime_type {
                                Some(v) => v.to_string(),
                                None => return Err(HandlerError::BadRequest("Couldn't get mime type".to_string())),
                            }
                        },
                        None => return Err(HandlerError::BadRequest("Couldn't get mime type".to_string())),
                    };

                    let bytes = field.bytes()
                        .await
                        .map_err(|_| {
                            HandlerError::BadRequest("Image has to be sent".to_string())
                        })?
                        .to_vec();
                    
                    (bytes, mime_type)
                },
                None => return Err(HandlerError::BadRequest("Image has to be sent".to_string())),
            };
            Some(bytes)
        },
        None => None,
    };

    let image_payload = match image_data {
        Some((image_bytes, mime_type)) => {
            let image = identity_grpc::Image {
                image: image_bytes,
                mime_type,
            };
            Some(image)
        },
        None => None,
    };

    let req = identity_grpc::UpdateAvatarReq {
        user_id,
        avatar: image_payload,
    };
    match identity_client.update_avatar(req).await {
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
            .route("/edit-profile", put(edit_my_profile))
            .route("/edit-avatar", put(edit_my_avatar))
            .layer(from_fn_with_state(auth_client, auth_middleware))
            .layer(CookieLayer::strict())
        )
        .with_state(UserState { identity_client })
}