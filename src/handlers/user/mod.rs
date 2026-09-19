use axum::{Extension, Json, Router, extract::{Path, State}, middleware::from_fn_with_state, response::{IntoResponse, Response}, routing::get};
use axum_cookie::CookieLayer;
use crate::{clients::{auth::AuthClient, identity::{IdentityClient, identity_grpc}}, handlers::{error::HandlerError, user::dto::{UserBasicProfileResponse, UserProfileResponse}}, middleware::auth::auth_middleware, util::resources::resource_key_to_url};

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
    
    let req = identity_grpc::UserId { user_id };
    match identity_client.get_full_profile(req).await {
        Ok(res) => {
            let profile = res.into_inner();
            
            let avatar_url = resource_key_to_url("avatar", profile.avatar_key);
            let banner_url = resource_key_to_url("banner", profile.banner_key);

            let res = UserProfileResponse {
                user_id: profile.user_id,
                username: profile.username,
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
async fn get_my_profile(
    State(state): State<UserState>,
    Extension(user_id): Extension<i64>,
) -> Result<Response, HandlerError> {
    let mut identity_client = state.identity_client;

    let req = identity_grpc::UserId { user_id };
    match identity_client.get_basic_profile(req).await {
        Ok(res) => {
            let res = res.into_inner();

            let avatar_url = resource_key_to_url("avatar", res.avatar_key);
            Ok(Json(UserBasicProfileResponse {
                user_id: res.user_id,
                username: res.username,
                avatar_url,
            }).into_response())
        },
        Err(_) => Err(HandlerError::NotFound),
    }
}

pub fn create_user_router(
    auth_client: AuthClient,
    identity_client: IdentityClient,
) -> Router {
    Router::new()
        .route("/{user_id}", get(get_user_profile))
        .merge(Router::new()
            .route("/me", get(get_my_profile))
            .layer(from_fn_with_state(auth_client, auth_middleware))
            .layer(CookieLayer::strict())
        )
        .with_state(UserState { identity_client })
}