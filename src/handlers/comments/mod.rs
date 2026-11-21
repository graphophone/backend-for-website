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
        music::{MusicClient, music_grpc},
    },
    handlers::{
        comments::dto::{AddCommentReq, CommentsPageQuery, UpdateCommentReq},
        error::HandlerError,
        music::dto::ShortMusicRes,
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
    pub music_client: MusicClient,
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

    comments_client
        .add_comment(req)
        .await
        .map_err(|_| HandlerError::InternalError)?
        .into_inner();

    Ok(StatusCode::OK.into_response())
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

    let req = comments_grpc::GetMusicCommentsReq {
        music_id,
        page_number: query.page_number,
        requester_id,
    };

    let comments_page = comments_client
        .get_music_comments(req)
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
            dto::MusicCommentEntry {
                comment_id: v.comment_id,
                profile: profile,
                content: v.content,
                created_at: v.created_at,
                likes_count: v.likes_count,
                is_liked: v.is_liked,
            }
        })
        .collect();

    Ok(Json(dto::MusicCommentsPage {
        total_pages: comments_page.total_pages,
        comments,
    })
    .into_response())
}

#[axum::debug_handler]
async fn get_user_comments(
    State(state): State<CommentsState>,
    Path(user_id): Path<i64>,
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
    let mut music_client = state.music_client;

    let req = comments_grpc::GetUserCommentsReq {
        user_id,
        page_number: query.page_number,
        requester_id,
    };

    let comments_page = comments_client
        .get_user_comments(req)
        .await
        .map_err(|_| HandlerError::InternalError)?
        .into_inner();

    let music_ids = comments_page.comments.iter().map(|v| v.music_id).collect();
    let req = music_grpc::MusicIds { music_ids };

    let mut music_map: Option<HashMap<_, _>> = match music_client.get_music_by_ids(req).await {
        Ok(v) => Some(
            v.into_inner()
                .music_entries
                .into_iter()
                .map(|v| (v.id, v))
                .collect(),
        ),
        Err(_) => None,
    };

    let comments = comments_page
        .comments
        .into_iter()
        .map(|v| {
            let music = match &mut music_map {
                Some(mus) => mus.get(&v.music_id).map(|m| ShortMusicRes {
                    id: m.id,
                    title: m.title.clone(),
                }),
                None => None,
            };
            dto::UserCommentEntry {
                comment_id: v.comment_id,
                music,
                content: v.content,
                created_at: v.created_at,
                likes_count: v.likes_count,
                is_liked: v.is_liked,
            }
        })
        .collect();

    Ok(Json(dto::UserCommentsPage {
        total_pages: comments_page.total_pages,
        comments,
    })
    .into_response())
}

pub fn create_comments_router(
    comments_client: CommentsClient,
    auth_client: AuthClient,
    identity_client: IdentityClient,
    music_client: MusicClient,
) -> Router {
    Router::new()
        .route("/music/{music_id}", get(get_music_comments))
        .route("/user/{user_id}", get(get_user_comments))
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
            music_client,
        })
        .layer(CookieLayer::strict())
}
