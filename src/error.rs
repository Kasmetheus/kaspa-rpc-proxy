use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Kaspa node error: {0}")]
    Kaspa(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

impl IntoResponse for RpcError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            RpcError::Connection(msg) => (StatusCode::BAD_GATEWAY, msg),
            RpcError::Kaspa(msg) => (StatusCode::BAD_REQUEST, msg),
            RpcError::InvalidResponse(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            RpcError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            RpcError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            RpcError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            error: message,
            code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for RpcError {
    fn from(err: anyhow::Error) -> Self {
        RpcError::Internal(err.to_string())
    }
}
