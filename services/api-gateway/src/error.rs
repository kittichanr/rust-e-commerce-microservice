use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("internal error: {0}")]
    Internal(String),

    #[error("grpc error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl ResponseError for GatewayError {
    fn status_code(&self) -> StatusCode {
        match self {
            GatewayError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            GatewayError::BadRequest(_) => StatusCode::BAD_REQUEST,
            GatewayError::Unauthorized => StatusCode::UNAUTHORIZED,
            GatewayError::Forbidden => StatusCode::FORBIDDEN,
            GatewayError::NotFound => StatusCode::NOT_FOUND,
            GatewayError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayError::Grpc(status) => match status.code() {
                tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
                tonic::Code::NotFound => StatusCode::NOT_FOUND,
                tonic::Code::AlreadyExists => StatusCode::CONFLICT,
                tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
                tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
                tonic::Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            GatewayError::Request(_) => StatusCode::BAD_GATEWAY,
            GatewayError::Jwt(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "error": self.to_string(),
            "status": self.status_code().as_u16()
        }))
    }
}
