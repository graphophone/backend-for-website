use crate::{
    clients::{
        comments::{CommentsClient, comments_grpc},
        music::{MusicClient, music_grpc::CheckUploaderReq},
    },
    handlers::error::HandlerError,
};
use axum::{
    Extension,
    extract::{Path, Request, State},
    middleware::Next,
    response::Response,
};

pub async fn music_owner_middleware(
    Extension(user_id): Extension<i64>,
    Path(music_id): Path<i64>,
    State(mut music_client): State<MusicClient>,
    req: Request,
    next: Next,
) -> Result<Response, HandlerError> {
    match music_client
        .check_uploader(CheckUploaderReq {
            music_id,
            uploader_id: user_id,
        })
        .await
    {
        Ok(_) => Ok(next.run(req).await),
        Err(_) => Err(HandlerError::Unauthorized),
    }
}

pub async fn comment_owner_middleware(
    Extension(user_id): Extension<i64>,
    State(mut comments_client): State<CommentsClient>,
    Path(comment_id): Path<i64>,
    req: Request,
    next: Next,
) -> Result<Response, HandlerError> {
    match comments_client
        .check_owner(comments_grpc::CheckOwnerReq {
            user_id,
            comment_id,
        })
        .await
    {
        Ok(_) => Ok(next.run(req).await),
        Err(_) => Err(HandlerError::Unauthorized),
    }
}
