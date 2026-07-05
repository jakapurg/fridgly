//! Web-layer error type and its mapping to HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use fridgly_domain::{DomainError, RepositoryError};

/// Everything a handler can fail with, mapped to an HTTP status on the way out.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Input failed domain validation → 400.
    #[error(transparent)]
    Validation(#[from] DomainError),

    /// Entity not found → 404.
    #[error("not found")]
    NotFound,

    /// Unexpected storage failure → 500.
    #[error("internal error")]
    Internal,
}

impl From<RepositoryError> for AppError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => AppError::NotFound,
            RepositoryError::Backend(msg) => {
                tracing::error!(error = %msg, "repository backend error");
                AppError::Internal
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_string(),
            ),
        };
        (status, body).into_response()
    }
}
