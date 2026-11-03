use crate::{
    clients::music::{MusicClient, music_grpc::CheckUploaderReq},
    handlers::error::HandlerError,
};
use axum::{
    Extension,
    extract::{Path, Request, State},
    middleware::Next,
    response::Response,
};

pub async fn owner_middleware(
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
