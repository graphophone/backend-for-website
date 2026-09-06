use axum::{http::StatusCode, response::{IntoResponse, Response}};

pub enum HandlerError {
    NotFound,
    InternalError,
    BadRequest,
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            Self::BadRequest => (StatusCode::INTERNAL_SERVER_ERROR, "Bad request"),
        };
        (status, body).into_response()
    }
}