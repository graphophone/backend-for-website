use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{delete, patch, post},
};
use axum_cookie::CookieLayer;

use crate::{
    clients::{
        auth::AuthClient,
        comments::{CommentsClient, comments_grpc},
    },
    handlers::{
        comments::dto::{AddCommentReq, UpdateCommentReq},
        error::HandlerError,
    },
    middleware::{auth::auth_middleware, owner::comment_owner_middleware},
};

pub mod dto;

#[axum::debug_handler]
async fn add_comment(
    State(mut comments_client): State<CommentsClient>,
    Extension(user_id): Extension<i64>,
    Path(music_id): Path<i64>,
    Json(req): Json<AddCommentReq>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::AddCommentReq {
        music_id,
        user_id,
        content: req.content,
    };

    comments_client
        .add_comment(req)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(StatusCode::OK.into_response())
}

#[axum::debug_handler]
async fn remove_comment(
    State(mut comments_client): State<CommentsClient>,
    Path(comment_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::RemoveCommentReq { comment_id };

    comments_client
        .remove_comment(req)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(StatusCode::OK.into_response())
}

#[axum::debug_handler]
async fn update_comment(
    State(mut comments_client): State<CommentsClient>,
    Path(comment_id): Path<i64>,
    Json(req): Json<UpdateCommentReq>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::UpdateCommentReq {
        comment_id,
        new_content: req.new_content,
    };

    comments_client
        .update_comment(req)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(StatusCode::OK.into_response())
}

#[axum::debug_handler]
async fn toggle_like(
    State(mut comments_client): State<CommentsClient>,
    Extension(user_id): Extension<i64>,
    Path(comment_id): Path<i64>,
) -> Result<Response, HandlerError> {
    let req = comments_grpc::ToggleLikeReq {
        comment_id,
        user_id,
    };

    comments_client
        .toggle_like(req)
        .await
        .map_err(|_| HandlerError::InternalError)?;

    Ok(StatusCode::OK.into_response())
}

pub fn create_comments_router(comments_client: CommentsClient, auth_client: AuthClient) -> Router {
    Router::new()
        .route("/{music_id}/add_comment", post(add_comment))
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
        .with_state(comments_client.clone())
        .layer(from_fn_with_state(auth_client, auth_middleware))
        .layer(CookieLayer::strict())
}
