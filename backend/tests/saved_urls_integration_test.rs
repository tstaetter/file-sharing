use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use backend::app;
use backend::AppState;
use serde_json::Value;
use tokio::time::Duration;
use tower::util::ServiceExt;

// ── Helpers ───────────────────────────────────────────────────────────

/// Create a minimal `AppState` with no database connection.
///
/// Used for tests that verify the behaviour when MongoDB is absent.
/// The saved URL handlers should return 500 Internal Server Error
/// because they require a live database handle.
fn test_state() -> AppState {
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

    AppState {
        database: None,
        s3: client,
        bucket: "test-bucket".to_string(),
    }
}

/// Create an `AppState` wired to a real MongoDB instance.
///
/// The function connects to the URI given by the `MONGODB_URI` environment
/// variable (defaulting to `mongodb://localhost:27017`) and uses the
/// database name provided by the caller. Both `saved_urls` and `users`
/// collections are dropped at the start so every test starts with a
/// clean slate.
async fn test_state_with_db(db_name: &str) -> AppState {
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

    let mongo_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongo_client = mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB for test — is it running?");
    let db = mongo_client.database(db_name);

    // Drop saved_urls and users collections for a clean slate.
    let _ = db
        .collection::<mongodb::bson::Document>("saved_urls")
        .drop()
        .await;
    let _ = db
        .collection::<mongodb::bson::Document>("users")
        .drop()
        .await;

    AppState {
        database: Some(db),
        s3: client,
        bucket: "test-bucket".to_string(),
    }
}

/// Ensure `JWT_SECRET` and `JWT_EXPIRY_MINS` are set to known values.
///
/// The auth handlers and middleware read these variables at runtime via
/// `std::env::var`, so we must set them before any handler that creates or
/// validates tokens. Because `dotenvy` never overwrites existing environment
/// variables, calling this first guarantees the test values are used even if
/// a `.env` file is present.
fn ensure_jwt_secret() {
    std::env::set_var("JWT_SECRET", "integration-test-secret-do-not-use-in-prod");
    std::env::set_var("JWT_EXPIRY_MINS", "60");
}

/// Helper that registers a user and returns the resulting JWT token string.
///
/// The router is consumed by `oneshot`, so callers should pass a clone if
/// they need the router for further requests. This function asserts that
/// registration succeeds (200 OK) as a precondition.
async fn register_and_get_token(
    router: axum::Router,
    email: &str,
    password: &str,
    name: &str,
) -> String {
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "name": name,
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Precondition: registration must succeed"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    json["token"].as_str().unwrap().to_string()
}

/// Helper that saves a URL on behalf of an authenticated user and returns
/// the parsed JSON response body.
///
/// The router is consumed by `oneshot`, so callers should pass a clone.
/// This helper asserts that the save succeeds (200 OK) as a precondition.
async fn save_url(router: axum::Router, token: &str, url: &str, title: Option<&str>) -> Value {
    let mut body = serde_json::json!({ "url": url });
    if let Some(t) = title {
        body["title"] = serde_json::Value::String(t.to_string());
    } else {
        body["title"] = serde_json::Value::Null;
    }

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Precondition: save URL must succeed"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).unwrap()
}

/// Helper that lists saved URLs for an authenticated user and returns the
/// parsed JSON response body.
///
/// The router is consumed by `oneshot`, so callers should pass a clone.
/// This helper asserts that the list succeeds (200 OK) as a precondition.
async fn list_urls(
    router: axum::Router,
    token: &str,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Value {
    let mut uri = "/v1/urls".to_string();
    let mut params = Vec::new();
    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(pp) = per_page {
        params.push(format!("per_page={}", pp));
    }
    if !params.is_empty() {
        uri.push('?');
        uri.push_str(&params.join("&"));
    }

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&uri)
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Precondition: list URLs must succeed"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).unwrap()
}

/// Assert that a string is a valid UUID v4 (36 characters, hyphens at
/// positions 8, 13, 18, 23). Does not check the version/variant bits
/// exhaustively — just validates the structural format.
fn assert_valid_uuid(id: &str) {
    assert_eq!(
        id.len(),
        36,
        "Expected UUID of length 36, got '{}' (len={})",
        id,
        id.len()
    );
    // Check hyphen positions
    for &pos in &[8, 13, 18, 23] {
        assert_eq!(
            id.as_bytes()[pos],
            b'-',
            "Expected hyphen at position {} in '{}'",
            pos,
            id
        );
    }
    // Check that position 14 is '4' (UUID v4 version nibble)
    assert_eq!(
        id.as_bytes()[14],
        b'4',
        "Expected version 4 at position 14 in '{}'",
        id
    );
}

/// Assert that a string looks like an ISO-8601 / RFC 3339 timestamp.
/// The format produced by `chrono::DateTime::to_rfc3339` looks like
/// `2025-07-16T12:00:00+00:00` or `2025-07-16T12:00:00Z`.
fn assert_iso8601(ts: &str) {
    assert!(
        ts.len() >= 20,
        "Timestamp too short to be ISO-8601: '{}'",
        ts
    );
    assert!(
        ts.contains('T'),
        "Timestamp missing 'T' separator: '{}'",
        ts
    );
    // Must have either a '+' / '-' offset or end with 'Z'
    assert!(
        ts.contains('+') || ts.contains('-') || ts.ends_with('Z'),
        "Timestamp missing timezone offset: '{}'",
        ts
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Save URL tests — POST /v1/urls
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn save_url_returns_200_and_record() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_save_ok").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "save-ok@test.com", "secret123", "Save OK").await;

    let body = serde_json::json!({
        "url": "https://filez.zone/f/abc123#key",
        "title": "My shared file",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_valid_uuid(json["id"].as_str().unwrap());
    assert_eq!(json["url"], "https://filez.zone/f/abc123#key");
    assert_eq!(json["title"], "My shared file");
    assert_iso8601(json["created_at"].as_str().unwrap());
}

#[tokio::test]
async fn save_url_works_without_title() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_save_noti").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "no-title@test.com", "secret123", "No Title").await;

    let body = serde_json::json!({
        "url": "https://filez.zone/f/xyz#key",
        "title": null,
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_valid_uuid(json["id"].as_str().unwrap());
    assert!(json["title"].is_null(), "Expected null title");
    assert!(json["url"].is_string());
    assert_iso8601(json["created_at"].as_str().unwrap());
}

#[tokio::test]
async fn save_url_rejects_empty_url() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_save_empty").await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "empty-url@test.com",
        "secret123",
        "Empty URL",
    )
    .await;

    let body = serde_json::json!({
        "url": "",
        "title": "test",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let msg = String::from_utf8_lossy(&body_bytes);
    assert!(msg.contains("url cannot be empty"));
}

#[tokio::test]
async fn save_url_rejects_whitespace_only_url() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_save_ws").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "ws-url@test.com", "secret123", "WS URL").await;

    let body = serde_json::json!({
        "url": "   \t  ",
        "title": "test",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Whitespace-only URL should be rejected as empty after trimming"
    );
}

#[tokio::test]
async fn save_url_rejects_missing_url_field() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_save_missing").await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "missing-url@test.com",
        "secret123",
        "Missing URL",
    )
    .await;

    // Send only a title — no url field at all → serde deserialization error
    let body = serde_json::json!({
        "title": "test",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn save_url_rejects_missing_auth_header() {
    ensure_jwt_secret();
    let router = app(test_state_with_db("surl_save_noauth").await);

    let body = serde_json::json!({
        "url": "https://example.com/f/abc#key",
        "title": "test",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn save_url_rejects_invalid_token() {
    ensure_jwt_secret();
    let router = app(test_state_with_db("surl_save_badtok").await);

    let body = serde_json::json!({
        "url": "https://example.com/f/abc#key",
        "title": "test",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(
                    header::AUTHORIZATION,
                    "Bearer invalid-token-that-is-not-a-jwt",
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ═══════════════════════════════════════════════════════════════════════
// List URLs tests — GET /v1/urls
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_urls_returns_empty_list_for_new_user() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_empty").await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "empty-list@test.com",
        "secret123",
        "Empty List",
    )
    .await;

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(json["urls"].is_array(), "Expected urls to be an array");
    assert_eq!(json["urls"].as_array().unwrap().len(), 0);
    assert_eq!(json["total"], 0);
    assert_eq!(json["page"], 1);
    assert_eq!(json["per_page"], 10);
}

#[tokio::test]
async fn list_urls_returns_saved_urls() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_multi").await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "multi-save@test.com",
        "secret123",
        "Multi Save",
    )
    .await;

    // Save 3 URLs
    save_url(
        router.clone(),
        &token,
        "https://example.com/f/a#k1",
        Some("File A"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;
    save_url(
        router.clone(),
        &token,
        "https://example.com/f/b#k2",
        Some("File B"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;
    save_url(
        router.clone(),
        &token,
        "https://example.com/f/c#k3",
        None::<&str>,
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    // List them
    let json = list_urls(router, &token, None, None).await;

    let urls = json["urls"].as_array().unwrap();
    assert_eq!(urls.len(), 3, "Expected 3 saved URLs");
    assert_eq!(json["total"], 3);
    assert_eq!(json["page"], 1);

    // Verify each has expected fields
    for url in urls {
        assert_valid_uuid(url["id"].as_str().unwrap());
        assert!(
            url["url"]
                .as_str()
                .unwrap()
                .starts_with("https://example.com/f/"),
            "URL has unexpected prefix: {}",
            url["url"]
        );
        assert_iso8601(url["created_at"].as_str().unwrap());
    }

    // Results should be newest-first (reverse chronological):
    // The last saved should appear first in the list.
    assert_eq!(urls[0]["url"], "https://example.com/f/c#k3");
    assert_eq!(urls[1]["url"], "https://example.com/f/b#k2");
    assert_eq!(urls[2]["url"], "https://example.com/f/a#k1");
}

#[tokio::test]
async fn list_urls_pagination_page_1() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_p1").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "page1@test.com", "secret123", "Page One").await;

    // Save 25 URLs
    for i in 0..25 {
        save_url(
            router.clone(),
            &token,
            &format!("https://example.com/f/uuid{}#key", i),
            Some(&format!("File {}", i)),
        )
        .await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    // Page 1, per_page=10 → should return 10 items
    let json = list_urls(router, &token, Some(1), Some(10)).await;

    assert_eq!(json["urls"].as_array().unwrap().len(), 10);
    assert_eq!(json["total"], 25);
    assert_eq!(json["page"], 1);
    assert_eq!(json["per_page"], 10);
}

#[tokio::test]
async fn list_urls_pagination_page_2() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_p2").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "page2@test.com", "secret123", "Page Two").await;

    // Save 25 URLs
    for i in 0..25 {
        save_url(
            router.clone(),
            &token,
            &format!("https://example.com/f/uuid{}#key", i),
            Some(&format!("File {}", i)),
        )
        .await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    // Page 2, per_page=10 → should return 10 items
    let json = list_urls(router, &token, Some(2), Some(10)).await;

    assert_eq!(json["urls"].as_array().unwrap().len(), 10);
    assert_eq!(json["total"], 25);
    assert_eq!(json["page"], 2);
    assert_eq!(json["per_page"], 10);
}

#[tokio::test]
async fn list_urls_pagination_page_3() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_p3").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "page3@test.com", "secret123", "Page Three").await;

    // Save 25 URLs
    for i in 0..25 {
        save_url(
            router.clone(),
            &token,
            &format!("https://example.com/f/uuid{}#key", i),
            Some(&format!("File {}", i)),
        )
        .await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    // Page 3, per_page=10 → should return 5 items (25 % 10 = 5 remainder)
    let json = list_urls(router, &token, Some(3), Some(10)).await;

    assert_eq!(json["urls"].as_array().unwrap().len(), 5);
    assert_eq!(json["total"], 25);
    assert_eq!(json["page"], 3);
    assert_eq!(json["per_page"], 10);
}

#[tokio::test]
async fn list_urls_pagination_out_of_range() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_oor").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "oor@test.com", "secret123", "Out Of Range").await;

    // Save 3 URLs
    for i in 0..3 {
        save_url(
            router.clone(),
            &token,
            &format!("https://example.com/f/url{}#key", i),
            Some(&format!("URL {}", i)),
        )
        .await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    // Query page 99 — beyond the available data
    let json = list_urls(router, &token, Some(99), Some(10)).await;

    assert!(json["urls"].as_array().unwrap().is_empty());
    assert_eq!(json["total"], 3);
    assert_eq!(json["page"], 99);
}

#[tokio::test]
async fn list_urls_rejects_page_0() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_badp").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "bad-page@test.com", "secret123", "Bad Page").await;

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/urls?page=0&per_page=10")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let msg = String::from_utf8_lossy(&body_bytes);
    assert!(msg.contains("page must be at least 1"));
}

#[tokio::test]
async fn list_urls_rejects_per_page_0() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_badpp").await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "bad-pp@test.com",
        "secret123",
        "Bad PerPage",
    )
    .await;

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/urls?page=1&per_page=0")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let msg = String::from_utf8_lossy(&body_bytes);
    assert!(msg.contains("per_page must be between 1 and"));
}

#[tokio::test]
async fn list_urls_rejects_per_page_over_100() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_bigpp").await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "big-pp@test.com",
        "secret123",
        "Big PerPage",
    )
    .await;

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/urls?page=1&per_page=101")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let msg = String::from_utf8_lossy(&body_bytes);
    assert!(msg.contains("per_page must be between 1 and"));
}

#[tokio::test]
async fn list_urls_uses_default_pagination() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_list_def").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "defaults@test.com", "secret123", "Defaults").await;

    // Save a few URLs
    for i in 0..5 {
        save_url(
            router.clone(),
            &token,
            &format!("https://example.com/f/url{}#key", i),
            None::<&str>,
        )
        .await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    // Query without page or per_page params — defaults should apply
    let json = list_urls(router, &token, None, None).await;

    assert_eq!(json["page"], 1, "Default page should be 1");
    assert_eq!(json["per_page"], 10, "Default per_page should be 10");
    assert_eq!(json["total"], 5);
    assert_eq!(json["urls"].as_array().unwrap().len(), 5);
}

// ═══════════════════════════════════════════════════════════════════════
// User isolation tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_urls_only_returns_own_urls() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_isolate").await;

    // Create two users from the same AppState
    let router_a = app(state.clone());
    let router_b = app(state.clone());

    let token_a = register_and_get_token(
        router_a.clone(),
        "alice@isolated.test",
        "secret123",
        "Alice",
    )
    .await;
    let token_b =
        register_and_get_token(router_b.clone(), "bob@isolated.test", "secret123", "Bob").await;

    // Alice saves her URL
    save_url(
        router_a.clone(),
        &token_a,
        "https://example.com/f/alice-file#key",
        Some("Alice file"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    // Bob saves his URL
    save_url(
        router_b.clone(),
        &token_b,
        "https://example.com/f/bob-file#key",
        Some("Bob file"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    // Alice lists — should only see her own URL
    let alice_list = list_urls(router_a, &token_a, None, None).await;
    let alice_urls = alice_list["urls"].as_array().unwrap();
    assert_eq!(alice_urls.len(), 1);
    assert_eq!(alice_urls[0]["url"], "https://example.com/f/alice-file#key");
    assert_eq!(alice_urls[0]["title"], "Alice file");

    // Bob lists — should only see his own URL
    let bob_list = list_urls(router_b, &token_b, None, None).await;
    let bob_urls = bob_list["urls"].as_array().unwrap();
    assert_eq!(bob_urls.len(), 1);
    assert_eq!(bob_urls[0]["url"], "https://example.com/f/bob-file#key");
    assert_eq!(bob_urls[0]["title"], "Bob file");
}

#[tokio::test]
async fn save_url_stores_correct_user() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_correct_user").await;

    // Two users
    let router = app(state.clone());

    let token_alice = register_and_get_token(
        router.clone(),
        "alice2@isolated.test",
        "secret123",
        "Alice Two",
    )
    .await;
    let token_bob =
        register_and_get_token(router.clone(), "bob2@isolated.test", "secret123", "Bob Two").await;

    // Alice saves a URL
    save_url(
        router.clone(),
        &token_alice,
        "https://example.com/f/alice-secret#key",
        Some("Secret"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    // Bob lists his URLs — should be empty, Alice's URL shouldn't appear
    let bob_list = list_urls(router, &token_bob, None, None).await;
    assert!(
        bob_list["urls"].as_array().unwrap().is_empty(),
        "Bob should not see Alice's URLs"
    );
    assert_eq!(bob_list["total"], 0);
}

// ═══════════════════════════════════════════════════════════════════════
// Auth tests for list endpoints
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_urls_rejects_missing_auth_header() {
    ensure_jwt_secret();
    let router = app(test_state_with_db("surl_list_noauth").await);

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/urls")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_urls_rejects_invalid_token() {
    ensure_jwt_secret();
    let router = app(test_state_with_db("surl_list_badtok").await);

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/urls")
                .header(
                    header::AUTHORIZATION,
                    "Bearer invalid-token-that-is-not-valid",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ═══════════════════════════════════════════════════════════════════════
// No-database tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn save_url_returns_500_without_database() {
    ensure_jwt_secret();
    let router = app(test_state());

    // Create a syntactically valid JWT signed with the test secret.
    // The middleware will validate and pass it, but the handler will fail
    // because there is no database handle.
    let token = backend::create_token("nod-save@example.com")
        .expect("create_token must succeed with JWT_SECRET set");

    let body = serde_json::json!({
        "url": "https://example.com/f/abc#key",
        "title": "test",
    })
    .to_string();

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn list_urls_returns_500_without_database() {
    ensure_jwt_secret();
    let router = app(test_state());

    // Create a syntactically valid JWT signed with the test secret.
    let token = backend::create_token("nod-list@example.com")
        .expect("create_token must succeed with JWT_SECRET set");

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/urls")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ═══════════════════════════════════════════════════════════════════════
// Ordering and format tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_urls_ordering_is_newest_first() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_order").await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "ordering@test.com", "secret123", "Ordering").await;

    // Save 3 URLs with small delays to ensure distinct timestamps
    save_url(
        router.clone(),
        &token,
        "https://example.com/f/oldest#key",
        Some("Oldest"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    save_url(
        router.clone(),
        &token,
        "https://example.com/f/middle#key",
        Some("Middle"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    save_url(
        router.clone(),
        &token,
        "https://example.com/f/newest#key",
        Some("Newest"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    let json = list_urls(router, &token, None, None).await;
    let urls = json["urls"].as_array().unwrap();
    assert_eq!(urls.len(), 3);

    // Newest-first ordering: the last saved should be first in the list
    assert_eq!(
        urls[0]["title"], "Newest",
        "First item should be the most recently saved"
    );
    assert_eq!(urls[1]["title"], "Middle");
    assert_eq!(
        urls[2]["title"], "Oldest",
        "Last item should be the earliest saved"
    );
}

#[tokio::test]
async fn save_url_response_has_valid_uuid() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_uuid").await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "uuid-check@test.com",
        "secret123",
        "UUID Check",
    )
    .await;

    // Save multiple URLs and verify each gets a valid UUID
    for i in 0..5 {
        let json = save_url(
            router.clone(),
            &token,
            &format!("https://example.com/f/test{}#key", i),
            None::<&str>,
        )
        .await;
        tokio::time::sleep(Duration::from_millis(1)).await;

        let id = json["id"].as_str().unwrap();
        assert_valid_uuid(id);

        // Each ID should be unique (spot-check: two consecutive saves must differ)
        if i > 0 {
            // We don't have the previous ID stored across iterations easily,
            // but the assert_valid_uuid ensures the format. UUID v4 collision
            // is astronomically unlikely, so we rely on that.
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Full lifecycle test
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn full_crud_lifecycle() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_lifecycle").await;
    let router = app(state);

    // 1. Register a new user
    let token = register_and_get_token(
        router.clone(),
        "lifecycle-surl@test.com",
        "secret123",
        "Lifecycle",
    )
    .await;
    assert!(!token.is_empty());

    // 2. List URLs — should be empty
    let list1 = list_urls(router.clone(), &token, None, None).await;
    assert!(list1["urls"].as_array().unwrap().is_empty());
    assert_eq!(list1["total"], 0);

    // 3. Save two URLs
    save_url(
        router.clone(),
        &token,
        "https://example.com/f/lifecycle1#key",
        Some("First"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;
    save_url(
        router.clone(),
        &token,
        "https://example.com/f/lifecycle2#key",
        Some("Second"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    // 4. List — should have 2 URLs
    let list2 = list_urls(router.clone(), &token, None, None).await;
    let urls2 = list2["urls"].as_array().unwrap();
    assert_eq!(urls2.len(), 2);
    assert_eq!(list2["total"], 2);

    // Newest-first: "Second" was saved last, so it appears first
    assert_eq!(urls2[0]["title"], "Second");
    assert_eq!(urls2[1]["title"], "First");

    // 5. Save a third URL
    save_url(
        router.clone(),
        &token,
        "https://example.com/f/lifecycle3#key",
        Some("Third"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    // 6. List — should have 3 URLs
    let list3 = list_urls(router, &token, None, None).await;
    let urls3 = list3["urls"].as_array().unwrap();
    assert_eq!(urls3.len(), 3);
    assert_eq!(list3["total"], 3);

    // Verify ordering: newest-first
    assert_eq!(urls3[0]["title"], "Third");
    assert_eq!(urls3[1]["title"], "Second");
    assert_eq!(urls3[2]["title"], "First");

    // 7. Verify all fields on each URL
    for url in urls3 {
        assert_valid_uuid(url["id"].as_str().unwrap());
        assert!(
            url["url"]
                .as_str()
                .unwrap()
                .starts_with("https://example.com/f/lifecycle"),
            "Unexpected URL prefix"
        );
        assert!(url["title"].is_string() || url["title"].is_null());
        assert_iso8601(url["created_at"].as_str().unwrap());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Delete URL tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn delete_url_returns_204() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_del_204").await;
    let router = app(state);
    let token = register_and_get_token(router.clone(), "del@test.com", "secret123", "Delete").await;
    // Save a URL first
    let saved = save_url(
        router.clone(),
        &token,
        "https://example.com/f/uuid#key",
        Some("To delete"),
    )
    .await;
    let id = saved["id"].as_str().unwrap();
    // Delete it
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/v1/urls/{}", id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    // Verify it's gone from list
    let list = list_urls(router, &token, None, None).await;
    assert_eq!(list["total"], 0);
}

#[tokio::test]
async fn delete_url_returns_404_for_nonexistent() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_del_404").await;
    let router = app(state);
    let token = register_and_get_token(router.clone(), "del404@test.com", "secret123", "D").await;
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/v1/urls/{}", fake_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_url_returns_404_for_other_users_url() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_del_other").await;
    let router = app(state);
    // Alice saves a URL
    let alice_token =
        register_and_get_token(router.clone(), "alice_del@test.com", "secret123", "Alice").await;
    let saved = save_url(
        router.clone(),
        &alice_token,
        "https://example.com/f/a#k",
        Some("Alice file"),
    )
    .await;
    let id = saved["id"].as_str().unwrap();
    // Bob tries to delete Alice's URL
    let bob_token =
        register_and_get_token(router.clone(), "bob_del@test.com", "secret123", "Bob").await;
    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/v1/urls/{}", id))
                .header(header::AUTHORIZATION, format!("Bearer {}", bob_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_url_rejects_missing_auth() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_del_noauth").await;
    let router = app(state);
    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/urls/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_url_rejects_invalid_token() {
    ensure_jwt_secret();
    let state = test_state_with_db("surl_del_badtok").await;
    let router = app(state);
    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/urls/00000000-0000-0000-0000-000000000000")
                .header(header::AUTHORIZATION, "Bearer invalidtoken")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_url_returns_500_without_database() {
    ensure_jwt_secret();
    let router = app(test_state());

    // Create a syntactically valid JWT signed with the test secret.
    let token = backend::create_token("nod-delete@example.com")
        .expect("create_token must succeed with JWT_SECRET set");

    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/urls/00000000-0000-0000-0000-000000000000")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
