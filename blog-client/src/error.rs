use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("Request error: {0}")]
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
    StatusError(#[from] Status)
}
