use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("user not found: {0}")]
    UserNotFound(Uuid),
    #[error("user already exists: {0}")]
    UserAlreadyExists(String),
    #[error("post not found: {0}")]
    PostNotFound(Uuid),
    #[error("forbidden")]
    Forbidden,
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ResponseError for DomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            DomainError::UserNotFound(_) | DomainError::PostNotFound(_) => StatusCode::NOT_FOUND,
            DomainError::Unauthorized => StatusCode::UNAUTHORIZED,
            DomainError::Forbidden => StatusCode::FORBIDDEN,
            DomainError::UserAlreadyExists(_) => StatusCode::CONFLICT,
            DomainError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = self.to_string();
        let details = match self {
            DomainError::PostNotFound(resource) | DomainError::UserNotFound(resource) => {
                Some(json!({ "resource": resource }))
            }
            DomainError::Forbidden => {
                Some(json!({ "message:": "you do not have permission to delete this post"}))
            }
            _ => None,
        };
        let body = ErrorBody {
            error: message.as_str(),
            details,
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}
