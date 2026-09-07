use axum::{http::StatusCode, response::{IntoResponse, Response}};

pub enum HandlerError {
    NotFound,
    InternalError,
    BadRequest(String),
    Unauthorized,
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, String::from("Not found")),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal server error")),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, String::from("Unauthorized")),
        };
        (status, body).into_response()
    }
}