use aws_sdk_s3::config::{BehaviorVersion, Region};
use axum_test::TestServer;
use backend::{AppState, app};

/// Build an S3 client suitable for presigned-URL generation without hitting
/// a real endpoint.  The `test-util` feature (enabled in `dev-dependencies`)
/// exposes `with_test_defaults()` which wires up a dummy credential provider
/// and other required config so that `.presigned()` can compute a signature
/// locally.  A region is still required for endpoint resolution during
/// presigned-URL construction.
fn test_s3_client() -> aws_sdk_s3::Client {
    let config = aws_sdk_s3::Config::builder()
        .with_test_defaults()
        .region(Region::new("us-east-1"))
        .behavior_version(BehaviorVersion::latest())
        .build();
    aws_sdk_s3::Client::from_conf(config)
}

#[tokio::test]
async fn test_sign_parts_multiple() {
    // ── Arrange ──────────────────────────────────────────────────────────
    let s3 = test_s3_client();

    let state = AppState {
        database: None,
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // ── Act ──────────────────────────────────────────────────────────────
    let response = test_server
        .post("/v1/sign-parts")
        .json(&serde_json::json!({
            "key": "uploads/test-file-id",
            "upload_id": "test-upload-id",
            "part_numbers": [1, 3, 5]
        }))
        .await;

    // ── Assert ───────────────────────────────────────────────────────────
    response.assert_status_ok();

    let parts: Vec<serde_json::Value> = response.json();
    assert_eq!(parts.len(), 3, "should return one URL per part number");

    for (i, part) in parts.iter().enumerate() {
        let expected_part_number = [1, 3, 5][i];

        assert_eq!(
            part["part_number"], expected_part_number,
            "part_number should match the requested part"
        );

        let url = part["url"].as_str().expect("url field should be a string");

        assert!(!url.is_empty(), "presigned URL should not be empty");

        // Basic structural assertions – the presigned URL must contain the
        // bucket, object key, upload ID, part number, and an AWS signature.
        assert!(
            url.contains("test-bucket"),
            "URL should reference the bucket: {url}"
        );
        // The key may be URL-encoded as `uploads%2Ftest-file-id` or kept
        // as `uploads/test-file-id` depending on the signer.
        assert!(
            url.contains("uploads%2Ftest-file-id") || url.contains("uploads/test-file-id"),
            "URL should contain the object key: {url}"
        );
        assert!(
            url.contains("uploadId=test-upload-id"),
            "URL should contain the upload ID: {url}"
        );
        assert!(
            url.contains(&format!("partNumber={expected_part_number}")),
            "URL should contain partNumber={expected_part_number}: {url}"
        );
        assert!(
            url.contains("X-Amz-Signature="),
            "URL should contain an AWS signature: {url}"
        );
        assert!(
            url.contains("X-Amz-Expires=3600"),
            "URL should expire in 3600 seconds (the handler's hard-coded expiry): {url}"
        );
    }
}

#[tokio::test]
async fn test_sign_parts_single() {
    // ── Arrange ──────────────────────────────────────────────────────────
    let s3 = test_s3_client();

    let state = AppState {
        database: None,
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // ── Act ──────────────────────────────────────────────────────────────
    let response = test_server
        .post("/v1/sign-parts")
        .json(&serde_json::json!({
            "key": "uploads/single-part",
            "upload_id": "upload-42",
            "part_numbers": [1]
        }))
        .await;

    // ── Assert ───────────────────────────────────────────────────────────
    response.assert_status_ok();

    let parts: Vec<serde_json::Value> = response.json();
    assert_eq!(parts.len(), 1, "single part_number should yield one URL");
    assert_eq!(parts[0]["part_number"], 1);

    let url = parts[0]["url"].as_str().expect("url should be a string");
    assert!(!url.is_empty(), "presigned URL should not be empty");
    assert!(
        url.contains("test-bucket"),
        "URL should reference the bucket: {url}"
    );
    assert!(
        url.contains("uploads%2Fsingle-part") || url.contains("uploads/single-part"),
        "URL should contain the object key: {url}"
    );
    assert!(
        url.contains("uploadId=upload-42"),
        "URL should contain the upload ID: {url}"
    );
    assert!(
        url.contains("partNumber=1"),
        "URL should contain partNumber=1: {url}"
    );
    assert!(
        url.contains("X-Amz-Signature="),
        "URL should contain an AWS signature: {url}"
    );
    assert!(
        url.contains("X-Amz-Expires=3600"),
        "URL should expire in 3600 seconds: {url}"
    );
}
