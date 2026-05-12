use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
use aws_smithy_mocks::{mock, mock_client};
use axum_test::TestServer;
use backend::{AppState, app};
use serde_json::json;

#[tokio::test]
async fn test_create_upload_without_content_type() {
    // Arrange: mock S3 create_multipart_upload, returning a known upload_id
    let rule = mock!(aws_sdk_s3::Client::create_multipart_upload)
        .match_requests(|req| req.key() == Some("uploads/test-file-id"))
        .then_output(|| {
            CreateMultipartUploadOutput::builder()
                .upload_id("mock-upload-id")
                .build()
        });

    let s3 = mock_client!(aws_sdk_s3, [&rule]);

    let state = AppState {
        database: None,
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // Act: POST without content_type
    let response = test_server
        .post("/v1/create-upload")
        .json(&json!({
            "file_id": "test-file-id"
        }))
        .await;

    // Assert: 200 OK, correct response body, S3 called exactly once
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["upload_id"], "mock-upload-id");
    assert_eq!(body["key"], "uploads/test-file-id");

    assert_eq!(rule.num_calls(), 1);
}

#[tokio::test]
async fn test_create_upload_with_content_type() {
    // Arrange: mock S3 create_multipart_upload
    let rule = mock!(aws_sdk_s3::Client::create_multipart_upload)
        .match_requests(|req| {
            req.key() == Some("uploads/test-file-id") && req.content_type() == Some("image/png")
        })
        .then_output(|| {
            CreateMultipartUploadOutput::builder()
                .upload_id("mock-upload-id-2")
                .build()
        });

    let s3 = mock_client!(aws_sdk_s3, [&rule]);

    let state = AppState {
        database: None,
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // Act: POST with content_type
    let response = test_server
        .post("/v1/create-upload")
        .json(&json!({
            "file_id": "test-file-id",
            "content_type": "image/png"
        }))
        .await;

    // Assert: 200 OK, correct response body, S3 called exactly once
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["upload_id"], "mock-upload-id-2");
    assert_eq!(body["key"], "uploads/test-file-id");

    assert_eq!(rule.num_calls(), 1);
}
