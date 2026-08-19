//! Application error type and its HTTP mapping.
//!
//! Java/Spring: `AppError` ≈ a set of exceptions, and `IntoResponse` ≈ a
//! `@ControllerAdvice` / `@ExceptionHandler` that turns each into a status code.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Every fallible handler / store call returns `Result<_, AppError>`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("invalid url")]
    InvalidUrl,
    #[error("unauthorized")]
    Unauthorized,
    #[error("storage error: {0}")]
    Storage(String),
}

impl IntoResponse for AppError {
    /// Map each variant to a `(StatusCode, message)` response.
    ///
    /// Java/Spring: the `@ExceptionHandler` methods returning `ResponseEntity`.
    /// Mapping: `NotFound → 404`, `InvalidUrl → 400`, `Unauthorized → 401`,
    /// `Storage → 500`. Return e.g. `(StatusCode::NOT_FOUND, "not found").into_response()`.
    fn into_response(self) -> Response {
        todo!("step 7: map AppError variants to (StatusCode, message) responses")
    }
}

/// Convenience conversion so `?` on a `sqlx` call yields `AppError::Storage`.
/// (Provided — this is plumbing, not part of the exercise.)
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Storage(e.to_string())
    }
}
