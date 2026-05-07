use aws_sdk_s3::operation::delete_object::DeleteObjectOutput;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::types::error::NoSuchKey;
use aws_smithy_mocks::{mock, mock_client};
use aws_smithy_types::byte_stream::ByteStream;
use axum::http::StatusCode;
use axum_test::TestServer;
use backend::{AppState, app};

#[tokio::test]
async fn test_download_happy_path() {
    // ── Arrange ──────────────────────────────────────────────────────────

    // Rule 1: get_object returns the encrypted blob + content_type metadata.
    let get_rule = mock!(aws_sdk_s3::Client::get_object)
        .match_requests(|req| req.key() == Some("uploads/test-file-id"))
        .then_output(|| {
            GetObjectOutput::builder()
                .body(ByteStream::from_static(b"encrypted-ciphertext"))
                .content_type("application/pdf")
                .build()
        });

    // Rule 2: delete_object succeeds (burn-after-reading).
    let delete_rule = mock!(aws_sdk_s3::Client::delete_object)
        .match_requests(|req| req.key() == Some("uploads/test-file-id"))
        .then_output(|| DeleteObjectOutput::builder().build());

    // Sequential mode: first call matches get_rule, second matches delete_rule.
    let s3 = mock_client!(aws_sdk_s3, [&get_rule, &delete_rule]);

    let state = AppState {
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // ── Act ──────────────────────────────────────────────────────────────

    let response = test_server.get("/v1/f/test-file-id").await;

    // ── Assert ───────────────────────────────────────────────────────────

    response.assert_status_ok();
    response.assert_header("cache-control", "no-store, no-cache, must-revalidate");
    response.assert_header("pragma", "no-cache");
    response.assert_header("expires", "0");
    response.assert_header("content-type", "application/octet-stream");
    response.assert_header("x-content-type", "application/pdf");

    let body_bytes = response.into_bytes();
    assert_eq!(
        body_bytes.as_ref(),
        b"encrypted-ciphertext",
        "response body should be the raw binary object data"
    );

    // Verify both S3 operations were exercised exactly once.
    assert_eq!(get_rule.num_calls(), 1);
    assert_eq!(delete_rule.num_calls(), 1);
}

#[tokio::test]
async fn test_download_with_chunk_size_metadata() {
    // ── Arrange ──────────────────────────────────────────────────────────

    let get_rule = mock!(aws_sdk_s3::Client::get_object)
        .match_requests(|req| req.key() == Some("uploads/chunked-file"))
        .then_output(|| {
            GetObjectOutput::builder()
                .body(ByteStream::from_static(b"chunked-encrypted-data"))
                .content_type("image/png")
                .metadata("chunk-size", "5242880")
                .build()
        });

    let delete_rule = mock!(aws_sdk_s3::Client::delete_object)
        .match_requests(|req| req.key() == Some("uploads/chunked-file"))
        .then_output(|| DeleteObjectOutput::builder().build());

    let s3 = mock_client!(aws_sdk_s3, [&get_rule, &delete_rule]);

    let state = AppState {
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // ── Act ──────────────────────────────────────────────────────────────

    let response = test_server.get("/v1/f/chunked-file").await;

    // ── Assert ───────────────────────────────────────────────────────────

    response.assert_status_ok();
    response.assert_header("content-type", "application/octet-stream");
    response.assert_header("x-content-type", "image/png");
    response.assert_header("x-chunk-size", "5242880");

    let body_bytes = response.into_bytes();
    assert_eq!(
        body_bytes.as_ref(),
        b"chunked-encrypted-data",
        "response body should be the raw binary object data"
    );

    assert_eq!(get_rule.num_calls(), 1);
    assert_eq!(delete_rule.num_calls(), 1);
}

#[tokio::test]
async fn test_download_not_found() {
    // ── Arrange ──────────────────────────────────────────────────────────

    // get_object returns NoSuchKey, simulating a file that doesn't exist.
    let get_rule = mock!(aws_sdk_s3::Client::get_object)
        .match_requests(|req| req.key() == Some("uploads/nonexistent"))
        .then_error(|| GetObjectError::NoSuchKey(NoSuchKey::builder().build()));

    let s3 = mock_client!(aws_sdk_s3, [&get_rule]);

    let state = AppState {
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // ── Act ──────────────────────────────────────────────────────────────

    let response = test_server.get("/v1/f/nonexistent").await;

    // ── Assert ───────────────────────────────────────────────────────────

    response.assert_status(StatusCode::NOT_FOUND);
    assert!(
        response.text().contains("file not found"),
        "response body should contain 'file not found'"
    );

    // Verify the get_object operation was attempted exactly once.
    // delete_object should never have been called because we returned early.
    assert_eq!(get_rule.num_calls(), 1);
}
