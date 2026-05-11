use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Errors that can occur during file download.
///
/// Each variant maps to an appropriate HTTP status code:
/// - [`NotFound`] → 404 (file doesn't exist or already downloaded)
/// - [`ServiceUnavailable`] → 503 (storage throttling or transient failure)
/// - [`FetchFailed`] → 500 (unexpected S3/R2 error)
/// - [`HeaderInvalid`] → 500 (failed to construct a response header)
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("file not found")]
    NotFound,

    #[error("storage service unavailable, try again later")]
    ServiceUnavailable,

    #[error("failed to fetch file from storage")]
    FetchFailed,

    #[error("invalid response header: {0}")]
    HeaderInvalid(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AbortUploadError {
    #[error("AbortUpload SDK error: {0}")]
    Sdk(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteUploadError {
    #[error("CompleteUpload SDK error: {0}")]
    Sdk(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SignPartsError {
    #[error("SignParts SDK error: {0}")]
    Sdk(String),
}

impl IntoResponse for SignPartsError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Sdk(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl IntoResponse for AbortUploadError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Sdk(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl IntoResponse for CompleteUploadError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Sdk(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl IntoResponse for DownloadError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::FetchFailed => StatusCode::INTERNAL_SERVER_ERROR,
            Self::HeaderInvalid(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
