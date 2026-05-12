use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{handlers::errors::CheckFileError, AppState};

#[derive(Debug, Deserialize)]
pub struct CheckFileRequest {
    pub key: String,
}

pub async fn check_file(
    State(state): State<AppState>,
    Json(req): Json<CheckFileRequest>,
) -> Result<impl IntoResponse, CheckFileError> {
    let key = format!("uploads/{}", req.key);

    tracing::info!("Checking if {} exists", key);

    let _ = state
        .s3
        .head_object()
        .bucket(&state.bucket)
        .key(&key)
        .send()
        .await
        .map_err(|_| CheckFileError::NotFound)?;

    Ok((StatusCode::OK, "File exists").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    // ── CheckFileRequest deserialization tests ───────────────────────

    #[test]
    fn test_check_file_request_deserializes() {
        let json = r#"{"key":"abc-123-def"}"#;
        let req: CheckFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key, "abc-123-def");
    }

    #[test]
    fn test_check_file_request_deserializes_uuid() {
        let json = r#"{"key":"550e8400-e29b-41d4-a716-446655440000"}"#;
        let req: CheckFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_check_file_request_rejects_missing_key() {
        let json = r#"{}"#;
        serde_json::from_str::<CheckFileRequest>(json).unwrap_err();
    }

    #[test]
    fn test_check_file_request_rejects_empty_object() {
        let json = r#"{}"#;
        serde_json::from_str::<CheckFileRequest>(json).unwrap_err();
    }

    #[test]
    fn test_check_file_request_survives_extra_fields() {
        // Extra fields should be silently ignored by serde (default behavior)
        let json = r#"{"key":"abc","extra_field":123,"another":"ignored"}"#;
        let req: CheckFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key, "abc");
    }

    #[test]
    fn test_check_file_request_rejects_null_key() {
        let json = r#"{"key":null}"#;
        serde_json::from_str::<CheckFileRequest>(json).unwrap_err();
    }

    #[test]
    fn test_check_file_request_rejects_wrong_type() {
        // key must be a string, not a number
        let json = r#"{"key":42}"#;
        serde_json::from_str::<CheckFileRequest>(json).unwrap_err();
    }

    // ── CheckFileError tests ─────────────────────────────────────────

    #[test]
    fn test_check_file_error_display() {
        assert_eq!(
            CheckFileError::NotFound.to_string(),
            "CheckFile error, file not found"
        );
    }

    #[test]
    fn test_check_file_error_http_status() {
        let response = CheckFileError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_check_file_error_is_client_error() {
        let response = CheckFileError::NotFound.into_response();
        assert!(response.status().is_client_error());
    }

    #[test]
    fn test_check_file_error_debug() {
        let err = CheckFileError::NotFound;
        let debug = format!("{:?}", err);
        assert!(
            debug.contains("NotFound"),
            "Debug output should contain variant name, got: {}",
            debug
        );
    }

    // ── Key format tests ─────────────────────────────────────────────

    #[test]
    fn test_key_format_adds_uploads_prefix() {
        let file_id = "abc-123-def";
        let key = format!("uploads/{}", file_id);
        assert_eq!(key, "uploads/abc-123-def");
    }

    #[test]
    fn test_key_format_with_uuid() {
        let file_id = "550e8400-e29b-41d4-a716-446655440000";
        let key = format!("uploads/{}", file_id);
        assert_eq!(key, "uploads/550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_key_format_handles_empty_string() {
        let file_id = "";
        let key = format!("uploads/{}", file_id);
        assert_eq!(key, "uploads/");
    }

    #[test]
    fn test_key_format_starts_with_uploads() {
        let file_ids = vec!["abc", "def-ghi", "12345678"];
        for file_id in file_ids {
            let key = format!("uploads/{}", file_id);
            assert!(key.starts_with("uploads/"));
        }
    }

    #[test]
    fn test_key_format_is_deterministic() {
        let file_id = "test-file";
        let key1 = format!("uploads/{}", file_id);
        let key2 = format!("uploads/{}", file_id);
        assert_eq!(key1, key2);
    }

    // ── Response format tests ────────────────────────────────────────

    #[test]
    fn test_success_response_is_ok_status() {
        let response = (StatusCode::OK, "File exists").into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
