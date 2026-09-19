use axum::{Json, Router, extract::{Path, State}, response::{IntoResponse, Response}, routing::get};

use crate::{clients::identity::{IdentityClient, identity_grpc}, handlers::{error::HandlerError, user::dto::UserProfileResponse}};

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
            let res = UserProfileResponse {
                user_id: profile.user_id,
                username: profile.username,
                avatar_url: profile.avatar_url,
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

pub fn create_user_router(
    identity_client: IdentityClient,
) -> Router {
    Router::new()
        .route("/{user_id}", get(get_user_profile))
        .with_state(UserState { identity_client })
}