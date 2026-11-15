use std::collections::HashMap;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use axum_cookie::{CookieLayer, CookieManager};

use crate::{
    clients::{
        auth::AuthClient,
        comments::{CommentsClient, comments_grpc},
        identity::{IdentityClient, identity_grpc},
    },
    handlers::{
        comments::dto::{AddCommentReq, CommentsPageQuery, UpdateCommentReq},
        error::HandlerError,
        user::dto::MyProfileRes,
    },
    middleware::{
        auth::{auth_middleware, get_user_id_from_cookies},
        owner::comment_owner_middleware,
    },
};

pub mod dto;

#[derive(Clone)]
struct CommentsState {
    pub comments_client: CommentsClient,
    pub auth_client: AuthClient,
    pub identity_client: IdentityClient,
}

#[axum::debug_handler]
async fn add_comment(
    State(state): State<CommentsState>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<AddCommentReq>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::AddCommentReq {
        music_id: req.music_id,
        user_id,
        content: req.content,
    };
    let mut comments_client = state.comments_client;
    let mut identity_client = state.identity_client;

    let new_comment = comments_client
        .add_comment(req)
        .await
        .map_err(|_| HandlerError::InternalError)?
        .into_inner();

    let req = identity_grpc::UserId {
        user_id: new_comment.commenter_id,
    };
    let profile = match identity_client.get_basic_profile(req).await {
        Ok(v) => Some(v.into_inner()),
        Err(_) => None,
    };

    Ok(Json(dto::CommentEntry {
        comment_id: new_comment.comment_id,
        profile: profile.map(|p| MyProfileRes {
            user_id: p.user_id,
            username: p.username,
            avatar_url: p.avatar_id,
        }),
        content: new_comment.content,
        created_at: new_comment.created_at,
    })
    .into_response())
}

#[axum::debug_handler]
async fn remove_comment(
    State(state): State<CommentsState>,
    Path(comment_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::RemoveCommentReq { comment_id };
    let mut comments_client = state.comments_client;

    comments_client
        .remove_comment(req)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(StatusCode::OK.into_response())
}

#[axum::debug_handler]
async fn update_comment(
    State(state): State<CommentsState>,
    Path(comment_id): Path<i64>,
    Json(req): Json<UpdateCommentReq>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::UpdateCommentReq {
        comment_id,
        new_content: req.new_content,
    };
    let mut comments_client = state.comments_client;

    comments_client
        .update_comment(req)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(StatusCode::OK.into_response())
}

#[axum::debug_handler]
async fn toggle_like(
    State(state): State<CommentsState>,
    Extension(user_id): Extension<i64>,
    Path(comment_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::ToggleLikeReq {
        comment_id,
        user_id,
    };
    let mut comments_client = state.comments_client;

    comments_client
        .toggle_like(req)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(StatusCode::OK.into_response())
}

#[axum::debug_handler]
async fn get_music_comments(
    State(state): State<CommentsState>,
    Path(music_id): Path<i64>,
    Query(query): Query<CommentsPageQuery>,
    cookies: CookieManager,
) -> Result<Response, HandlerError> {
    let requester_id: Result<i64, HandlerError> =
        get_user_id_from_cookies(state.auth_client, cookies).await;
    let requester_id = match requester_id {
        Ok(v) => Some(v),
        Err(_) => None,
    };
    let mut comments_client = state.comments_client;
    let mut identity_client = state.identity_client;

    let req = comments_grpc::GetCommentsReq {
        music_id,
        page_number: query.page_number,
        requester_id,
    };

    let comments_page = comments_client
        .get_comments(req)
        .await
        .map_err(|_| HandlerError::InternalError)?
        .into_inner();

    let user_ids = comments_page
        .comments
        .iter()
        .map(|v| v.commenter_id)
        .collect();
    let req = identity_grpc::UserIds { user_ids };

    let mut profiles: Option<HashMap<_, _>> = match identity_client.get_profiles(req).await {
        Ok(v) => Some(
            v.into_inner()
                .profiles
                .into_iter()
                .map(|v| (v.user_id, v))
                .collect(),
        ),
        Err(_) => None,
    };

    let comments = comments_page
        .comments
        .into_iter()
        .map(|v| {
            let profile = match &mut profiles {
                Some(prof) => prof.get(&v.commenter_id).map(|p| MyProfileRes {
                    user_id: p.user_id,
                    username: p.username.clone(),
                    avatar_url: p.avatar_id.clone(),
                }),
                None => None,
            };
            dto::CommentEntry {
                comment_id: v.comment_id,
                profile: profile,
                content: v.content,
                created_at: v.created_at,
            }
        })
        .collect();

    Ok(Json(dto::CommentsPage {
        total_pages: comments_page.total_pages,
        comments,
    })
    .into_response())
}

pub fn create_comments_router(
    comments_client: CommentsClient,
    auth_client: AuthClient,
    identity_client: IdentityClient,
) -> Router {
    Router::new()
        .route("/music/{music_id}", get(get_music_comments))
        .merge(
            Router::new()
                .route("/", post(add_comment))
                .route("/{comment_id}/like", patch(toggle_like))
                .merge(
                    Router::new()
                        .route("/{comment_id}", delete(remove_comment))
                        .route("/{comment_id}", patch(update_comment))
                        .layer(from_fn_with_state(
                            comments_client.clone(),
                            comment_owner_middleware,
                        )),
                )
                .layer(from_fn_with_state(auth_client.clone(), auth_middleware)),
        )
        .with_state(CommentsState {
            comments_client,
            auth_client,
            identity_client,
        })
        .layer(CookieLayer::strict())
}
