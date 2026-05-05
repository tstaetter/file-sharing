use aws_sdk_s3::operation::abort_multipart_upload::AbortMultipartUploadOutput;
use aws_smithy_mocks::{mock, mock_client};
use axum_test::TestServer;
use backend::{AppState, app};
use serde_json::json;

#[tokio::test]
async fn test_abort_upload() {
    // Arrange: create a mock rule that matches the expected S3 request
    let rule = mock!(aws_sdk_s3::Client::abort_multipart_upload)
        .match_requests(|req| {
            req.key() == Some("uploads/test-file-id") && req.upload_id() == Some("test-upload-id")
        })
        .then_output(|| AbortMultipartUploadOutput::builder().build());

    // Create a mocked S3 client and wire it into application state
    let s3 = mock_client!(aws_sdk_s3, [&rule]);

    let state = AppState {
        s3,
        bucket: "test-bucket".to_string(),
    };

    let test_server = TestServer::new(app(state));

    // Act: POST to the abort-upload endpoint
    let response = test_server
        .post("/abort-upload")
        .json(&json!({
            "key": "uploads/test-file-id",
            "upload_id": "test-upload-id"
        }))
        .await;

    // Assert: endpoint responds 200 OK and the mock rule was exercised exactly once
    response.assert_status_ok();
    assert_eq!(rule.num_calls(), 1);
}
