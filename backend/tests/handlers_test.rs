use backend::*;
use serde_json;

// ── create_upload ──────────────────────────────────────────────────

#[test]
fn create_request_deserializes_minimal() {
    let json = r#"{"file_id":"abc-123"}"#;
    let req: CreateRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.file_id, "abc-123");
    assert!(req.content_type.is_none());
    assert!(req.chunk_size.is_none());
}

#[test]
fn create_request_deserializes_with_content_type() {
    let json = r#"{"file_id":"abc-123","content_type":"image/png"}"#;
    let req: CreateRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.file_id, "abc-123");
    assert_eq!(req.content_type.as_deref(), Some("image/png"));
    assert!(req.chunk_size.is_none());
}

#[test]
fn create_request_deserializes_with_chunk_size() {
    let json = r#"{"file_id":"abc-123","chunk_size":6291456}"#;
    let req: CreateRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.file_id, "abc-123");
    assert_eq!(req.chunk_size, Some(6291456));
}

#[test]
fn create_request_deserializes_full() {
    let json = r#"{"file_id":"abc-123","content_type":"application/pdf","chunk_size":5242880}"#;
    let req: CreateRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.file_id, "abc-123");
    assert_eq!(req.content_type.as_deref(), Some("application/pdf"));
    assert_eq!(req.chunk_size, Some(5242880));
}

#[test]
fn create_request_rejects_missing_file_id() {
    let json = r#"{"content_type":"image/png"}"#;
    let err = serde_json::from_str::<CreateRequest>(json).unwrap_err();
    assert!(
        err.to_string().contains("file_id"),
        "Error should mention missing file_id, got: {}",
        err
    );
}

#[test]
fn create_request_rejects_empty_object() {
    let json = r#"{}"#;
    let err = serde_json::from_str::<CreateRequest>(json).unwrap_err();
    assert!(
        err.to_string().contains("file_id"),
        "Error should mention missing file_id, got: {}",
        err
    );
}

#[test]
fn create_response_serializes_correctly() {
    let resp = CreateResponse {
        upload_id: "upload-123".to_string(),
        key: "uploads/abc-123".to_string(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["upload_id"], "upload-123");
    assert_eq!(parsed["key"], "uploads/abc-123");
}

#[test]
fn create_response_has_expected_keys() {
    let resp = CreateResponse {
        upload_id: "some-id".to_string(),
        key: "uploads/some-id".to_string(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(obj.len(), 2, "CreateResponse should have exactly 2 keys");
    assert!(obj.contains_key("upload_id"));
    assert!(obj.contains_key("key"));
}

// ── sign_parts ─────────────────────────────────────────────────────

#[test]
fn sign_parts_request_deserializes() {
    let json = r#"{"key":"uploads/abc","upload_id":"up-1","part_numbers":[1,2,3]}"#;
    let req: SignPartsRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.key, "uploads/abc");
    assert_eq!(req.upload_id, "up-1");
    assert_eq!(req.part_numbers, vec![1, 2, 3]);
}

#[test]
fn sign_parts_request_allows_empty_part_numbers() {
    let json = r#"{"key":"uploads/abc","upload_id":"up-1","part_numbers":[]}"#;
    let req: SignPartsRequest = serde_json::from_str(json).unwrap();
    assert!(req.part_numbers.is_empty());
}

#[test]
fn sign_parts_request_rejects_missing_part_numbers() {
    let json = r#"{"key":"uploads/abc","upload_id":"up-1"}"#;
    let err = serde_json::from_str::<SignPartsRequest>(json).unwrap_err();
    assert!(
        err.to_string().contains("part_numbers"),
        "Error should mention part_numbers, got: {}",
        err
    );
}

#[test]
fn signed_part_serializes_correctly() {
    let part = SignedPart {
        part_number: 1,
        url: "https://example.com/presigned".to_string(),
    };
    let json = serde_json::to_value(&part).unwrap();
    assert_eq!(json["part_number"], 1);
    assert_eq!(json["url"], "https://example.com/presigned");
}

// ── complete_upload ────────────────────────────────────────────────

#[test]
fn complete_request_deserializes() {
    let json = r#"{
        "key": "uploads/abc",
        "upload_id": "up-1",
        "parts": [
            {"part_number": 1, "etag": "\"abc123\""},
            {"part_number": 2, "etag": "\"def456\""}
        ]
    }"#;
    let req: CompleteRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.key, "uploads/abc");
    assert_eq!(req.upload_id, "up-1");
    assert_eq!(req.parts.len(), 2);
    assert_eq!(req.parts[0].part_number, 1);
    assert_eq!(req.parts[0].etag, "\"abc123\"");
    assert_eq!(req.parts[1].part_number, 2);
    assert_eq!(req.parts[1].etag, "\"def456\"");
}

#[test]
fn complete_request_allows_empty_parts() {
    let json = r#"{"key": "uploads/abc", "upload_id": "up-1", "parts": []}"#;
    let req: CompleteRequest = serde_json::from_str(json).unwrap();
    assert!(req.parts.is_empty());
}

#[test]
fn complete_request_rejects_missing_parts() {
    let json = r#"{"key": "uploads/abc", "upload_id": "up-1"}"#;
    serde_json::from_str::<CompleteRequest>(json).unwrap_err();
}

// ── abort_upload ───────────────────────────────────────────────────

#[test]
fn abort_request_deserializes() {
    let json = r#"{"key":"uploads/abc","upload_id":"up-1"}"#;
    let req: AbortRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.key, "uploads/abc");
    assert_eq!(req.upload_id, "up-1");
}

#[test]
fn abort_request_rejects_missing_key() {
    let json = r#"{"upload_id":"up-1"}"#;
    serde_json::from_str::<AbortRequest>(json).unwrap_err();
}

#[test]
fn abort_request_rejects_missing_upload_id() {
    let json = r#"{"key":"uploads/abc"}"#;
    serde_json::from_str::<AbortRequest>(json).unwrap_err();
}

// ── Key format convention ──────────────────────────────────────────

#[test]
fn upload_key_starts_with_uploads_prefix() {
    // All object keys in R2 must be under the "uploads/" prefix.
    // This test documents that convention.
    let file_id = "test-uuid-123";
    let key = format!("uploads/{}", file_id);
    assert!(key.starts_with("uploads/"));
    assert_eq!(key, "uploads/test-uuid-123");
}

#[test]
fn upload_key_contains_file_id() {
    let file_id = "abc-def-456";
    let key = format!("uploads/{}", file_id);
    assert!(key.ends_with(file_id));
}

// ── Health response ────────────────────────────────────────────────

#[test]
fn health_response_serializes_to_ok() {
    let resp = HealthResponse { status: "ok" };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json.as_object().unwrap().len(), 1);
}

// ── Serialization round-trips ──────────────────────────────────────

#[test]
fn create_request_round_trip() {
    let original = CreateRequest {
        file_id: "test-id".to_string(),
        content_type: Some("text/plain".to_string()),
        chunk_size: Some(1048576),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: CreateRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.file_id, original.file_id);
    assert_eq!(parsed.content_type, original.content_type);
    assert_eq!(parsed.chunk_size, original.chunk_size);
}

#[test]
fn create_request_survives_extra_fields() {
    // Clients may send extra fields we don't care about — this
    // should not fail deserialization.
    let json = r#"{"file_id":"abc","extra_field":"ignored","another":42}"#;
    let req: CreateRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.file_id, "abc");
}
