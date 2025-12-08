use reqwest::StatusCode;
use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP error {status}: {message}")]
    Http { status: StatusCode, message: String },
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::transport::Error),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Status error: {0}")]
    StatusError(#[from] Status),
}

// В любом месте, где ты обрабатываешь HTTP-ответ
impl BlogClientError {
    pub async fn from_http_response(resp: reqwest::Response) -> Self {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read body".into());

        BlogClientError::Http {
            status,
            message: text,
        }
    }
}
