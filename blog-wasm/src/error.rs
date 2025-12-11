use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },
    #[error(transparent)]
    RequestError(#[from] gloo_net::Error),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}
