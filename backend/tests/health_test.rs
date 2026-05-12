use axum::body::Body;
use axum::http::{Request, StatusCode};
use backend::app;
use tower::util::ServiceExt;

/// Creates a minimal AppState with a dummy S3 client for tests
/// that exercise the router but never actually call S3 operations.
fn test_state() -> backend::AppState {
    let credentials = aws_sdk_s3::config::Credentials::new(
        "test-access-key",
        "test-secret-key",
        None,
        None,
        "test-provider",
    );

    let config = aws_sdk_s3::Config::builder()
        .credentials_provider(credentials)
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .endpoint_url("http://localhost:9000")
        .force_path_style(true)
        .build();

    let client = aws_sdk_s3::Client::from_conf(config);

    backend::AppState {
        database: None,
        s3: client,
        bucket: "test-bucket".to_string(),
    }
}

#[tokio::test]
async fn health_returns_200_and_ok_json() {
    let router = app(test_state());

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn health_returns_json_content_type() {
    let router = app(test_state());

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok());

    assert!(
        content_type.is_some_and(|ct| ct.contains("application/json")),
        "Expected application/json content type, got: {:?}",
        content_type
    );
}

#[tokio::test]
async fn health_accepts_get_method() {
    let router = app(test_state());

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_rejects_post_method() {
    let router = app(test_state());

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // POST to a GET-only route should return 405 Method Not Allowed
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn health_endpoint_does_not_require_state() {
    // The health handler function takes no State extractor,
    // so calling it directly works without any AppState.
    let (status, json) = backend::health().await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json.status, "ok");
}

#[tokio::test]
async fn health_endpoint_not_at_v1_prefix() {
    // /v1/health should NOT exist — the health route is at /health,
    // outside the /v1 nest.
    let router = app(test_state());

    let response = router
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
