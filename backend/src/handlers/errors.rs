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

#[derive(Debug, thiserror::Error)]
pub enum CreateUploadError {
    #[error("CreateUpload response builder error: {0}")]
    ResponseBuilder(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CheckFileError {
    #[error("CheckFile error, file not found")]
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("user with this email already exists")]
    UserExists,

    #[error("invalid email or password")]
    InvalidCredentials,

    #[error("user not found")]
    NotFound,

    #[error("failed to hash password")]
    HashError,

    #[error("failed to create JWT token: {0}")]
    JwtToken(String),

    #[error("invalid or expired token: {0}")]
    JwtValidation(String),

    #[error("database error: {0}")]
    Database(String),
}

impl IntoResponse for CheckFileError {
    fn into_response(self) -> Response {
        (StatusCode::NOT_FOUND, self.to_string()).into_response()
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::UserExists => StatusCode::CONFLICT,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::HashError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtToken(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtValidation(_) => StatusCode::UNAUTHORIZED,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl IntoResponse for CreateUploadError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::ResponseBuilder(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    // ── DownloadError tests ──────────────────────────────────────────

    #[test]
    fn download_error_display() {
        assert_eq!(DownloadError::NotFound.to_string(), "file not found");
        assert_eq!(
            DownloadError::ServiceUnavailable.to_string(),
            "storage service unavailable, try again later"
        );
        assert_eq!(
            DownloadError::FetchFailed.to_string(),
            "failed to fetch file from storage"
        );
        assert_eq!(
            DownloadError::HeaderInvalid("X-Bad".into()).to_string(),
            "invalid response header: X-Bad"
        );
    }

    #[test]
    fn download_error_http_status() {
        let not_found = DownloadError::NotFound.into_response();
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);

        let unavailable = DownloadError::ServiceUnavailable.into_response();
        assert_eq!(unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);

        let fetch_failed = DownloadError::FetchFailed.into_response();
        assert_eq!(fetch_failed.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let header_invalid = DownloadError::HeaderInvalid("X-Test".into()).into_response();
        assert_eq!(header_invalid.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn download_error_debug_contains_message() {
        let err = DownloadError::NotFound;
        let debug = format!("{:?}", err);
        assert!(
            debug.contains("NotFound"),
            "Debug output should contain variant name, got: {}",
            debug
        );
    }

    // ── AbortUploadError tests ───────────────────────────────────────

    #[test]
    fn abort_upload_error_display() {
        assert_eq!(
            AbortUploadError::Sdk("timeout".into()).to_string(),
            "AbortUpload SDK error: timeout"
        );
    }

    #[test]
    fn abort_upload_error_http_status() {
        let resp = AbortUploadError::Sdk("any".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ── CompleteUploadError tests ────────────────────────────────────

    #[test]
    fn complete_upload_error_display() {
        assert_eq!(
            CompleteUploadError::Sdk("invalid part".into()).to_string(),
            "CompleteUpload SDK error: invalid part"
        );
    }

    #[test]
    fn complete_upload_error_http_status() {
        let resp = CompleteUploadError::Sdk("any".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ── SignPartsError tests ─────────────────────────────────────────

    #[test]
    fn sign_parts_error_display() {
        assert_eq!(
            SignPartsError::Sdk("throttled".into()).to_string(),
            "SignParts SDK error: throttled"
        );
    }

    #[test]
    fn sign_parts_error_http_status() {
        let resp = SignPartsError::Sdk("any".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ── Round-trip: all errors produce 4xx/5xx status codes ──────────

    /// Helper that asserts a value implements IntoResponse and returns a 4xx or 5xx status.
    fn assert_error_status(response: Response) {
        assert!(
            response.status().is_client_error() || response.status().is_server_error(),
            "Expected error status (4xx/5xx), got {}",
            response.status()
        );
    }

    #[test]
    fn all_errors_into_response_returns_valid_http() {
        assert_error_status(DownloadError::NotFound.into_response());
        assert_error_status(DownloadError::ServiceUnavailable.into_response());
        assert_error_status(DownloadError::FetchFailed.into_response());
        assert_error_status(DownloadError::HeaderInvalid("X-Foo".into()).into_response());
        assert_error_status(AbortUploadError::Sdk("sdk err".into()).into_response());
        assert_error_status(CompleteUploadError::Sdk("sdk err".into()).into_response());
        assert_error_status(SignPartsError::Sdk("sdk err".into()).into_response());
    }

    #[test]
    fn download_error_header_invalid_stores_the_header_name() {
        let err = DownloadError::HeaderInvalid("X-Custom-Header".into());
        assert!(err.to_string().contains("X-Custom-Header"));
    }
}
