use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use backend::app;
use backend::AppState;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::Value;
use tower::util::ServiceExt;

// ── Helpers ───────────────────────────────────────────────────────────

/// Create a minimal `AppState` with no database connection.
///
/// Used for tests that verify the behaviour when MongoDB is absent.
/// All three auth endpoints should return 500 Internal Server Error
/// because the handlers require a live database handle.
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
/// database `filez_zone_auth_test`. The `users` collection is dropped at
/// the start so every test starts with a clean slate.
async fn test_state_with_db() -> AppState {
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
    let db = mongo_client.database("filez_zone_auth_test");

    // Clean slate: drop the users collection so each test is isolated.
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
/// The auth handlers read these variables at runtime via `std::env::var`,
/// so we must set them before any handler that creates or validates tokens.
/// Because `dotenvy` never overwrites existing environment variables,
/// calling this first guarantees the test values are used even if a `.env`
/// file is present.
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

// ── Registration ──────────────────────────────────────────────────────

#[tokio::test]
async fn register_returns_200_and_token() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"test@example.com","password":"secret123","name":"Test User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["token"].is_string(), "Expected a token string");
    assert_eq!(json["user"]["email"], "test@example.com");
    assert_eq!(json["user"]["name"], "Test User");
    // password_hash must never leak to the client
    assert!(
        json["user"]["password_hash"].is_null(),
        "password_hash must not be exposed"
    );
}

#[tokio::test]
async fn register_rejects_duplicate_email() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    // `oneshot` consumes the router, but `Router` implements `Clone`.
    // We clone before the first request so both calls share the same
    // underlying state (i.e. the same MongoDB handle).
    let second_call_router = router.clone();

    // First registration succeeds
    let token = register_and_get_token(router, "dup@example.com", "secret123", "First").await;
    assert!(!token.is_empty());

    // Second registration with the same email → 409 Conflict
    let response = second_call_router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"dup@example.com","password":"secret123","name":"Second"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let msg = String::from_utf8_lossy(&body);
    assert!(msg.contains("already exists"));
}

#[tokio::test]
async fn register_rejects_empty_email() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"","password":"secret123","name":"Test User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_rejects_empty_password() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"test@example.com","password":"","name":"Test User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_rejects_empty_name() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"test@example.com","password":"secret123","name":""}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_rejects_missing_fields() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    // Missing `email` field entirely → Serde deserialization error → 422
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"password":"secret123","name":"Test User"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn register_returns_json_content_type() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"ct@example.com","password":"secret123","name":"CT Test"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    assert!(
        content_type.is_some_and(|ct| ct.contains("application/json")),
        "Expected application/json content type, got: {:?}",
        content_type
    );
}

// ── Login ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn login_returns_token_for_valid_credentials() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    // Register first via the helper
    let _token = register_and_get_token(
        router.clone(),
        "login-test@example.com",
        "secret123",
        "Tester",
    )
    .await;

    // Now login with the same credentials
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"login-test@example.com","password":"secret123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json["token"].is_string(),
        "Expected a token in login response"
    );
    assert_eq!(json["user"]["email"], "login-test@example.com");
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    let _token = register_and_get_token(
        router.clone(),
        "wrong-pass@example.com",
        "correct",
        "Person",
    )
    .await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"wrong-pass@example.com","password":"wrong"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_nonexistent_user() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"nobody@example.com","password":"whatever"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_empty_email() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"email":"","password":"secret123"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_empty_password() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"email":"test@example.com","password":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── Token validation ──────────────────────────────────────────────────

#[tokio::test]
async fn generated_token_is_valid_jwt() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    let token =
        register_and_get_token(router, "jwt-test@example.com", "secret123", "JWT Tester").await;

    // A JWT has three dot-separated parts: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "JWT must have exactly 3 dot-separated parts"
    );

    // Decode and validate with the same secret to verify the token is legitimate
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token_data = decode::<backend::Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .expect("Token must decode successfully");

    assert_eq!(token_data.claims.sub, "jwt-test@example.com");
    assert!(
        token_data.claims.exp > 0,
        "exp claim must be present and positive"
    );
    assert!(
        token_data.claims.iat > 0,
        "iat claim must be present and positive"
    );
    // exp should be after iat (token expires in the future relative to issuance)
    assert!(
        token_data.claims.exp > token_data.claims.iat,
        "exp must be after iat"
    );
}

#[tokio::test]
async fn token_can_be_used_for_authentication() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "auth-test@example.com",
        "secret123",
        "Auth Tester",
    )
    .await;

    // Use the token to access the protected DELETE endpoint
    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // The token is valid and the endpoint exists → 204 No Content on success
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

// ── Delete account ────────────────────────────────────────────────────

#[tokio::test]
async fn delete_returns_204_with_valid_token() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    let token = register_and_get_token(
        router.clone(),
        "delete-me@example.com",
        "secret123",
        "Delete Me",
    )
    .await;

    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify the body is empty (as per 204 No Content convention)
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(body.is_empty(), "204 response should have an empty body");
}

#[tokio::test]
async fn delete_rejects_missing_auth_header() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                // No Authorization header at all
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_rejects_invalid_token() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                .header(
                    header::AUTHORIZATION,
                    "Bearer invalid-token-that-is-not-a-jwt",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_rejects_malformed_auth_header() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                // "NotBearer" instead of "Bearer" — the middleware requires the
                // header to start with "Bearer " exactly
                .header(header::AUTHORIZATION, "NotBearer some-token-value")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn deleted_user_cannot_login() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    let token =
        register_and_get_token(router.clone(), "gone@example.com", "secret123", "Gone User").await;

    // Delete the user
    let delete_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Now try to log in with the deleted user's credentials → 401
    let login_response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"gone@example.com","password":"secret123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);
}

// ── No-database tests ─────────────────────────────────────────────────

#[tokio::test]
async fn register_returns_500_without_database() {
    ensure_jwt_secret();
    let router = app(test_state());

    // Send valid non-empty credentials — the handler passes input validation
    // but then fails when it tries to access the missing database handle.
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"nod@example.com","password":"secret123","name":"No DB"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn login_returns_500_without_database() {
    ensure_jwt_secret();
    let router = app(test_state());

    // Send valid non-empty credentials — the handler passes input validation
    // but then fails when it tries to access the missing database handle.
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"nod@example.com","password":"secret123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn delete_returns_500_without_database() {
    ensure_jwt_secret();
    let router = app(test_state());

    // Create a syntactically valid JWT signed with the test secret.
    // The middleware will validate and pass it, but the handler will fail
    // because there is no database handle.
    let token = backend::create_token("fake@example.com")
        .expect("create_token must succeed with JWT_SECRET set");

    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ── Full lifecycle ────────────────────────────────────────────────────

#[tokio::test]
async fn full_auth_lifecycle() {
    ensure_jwt_secret();
    let state = test_state_with_db().await;
    let router = app(state);

    // 1. Register a new user
    let token = register_and_get_token(
        router.clone(),
        "lifecycle@example.com",
        "secret123",
        "Lifecycle User",
    )
    .await;
    assert!(!token.is_empty());

    // 2. Delete the user with the token
    let delete_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/delete")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // 3. Try to log in with the same credentials → must fail
    let login_response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"email":"lifecycle@example.com","password":"secret123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);
}

// ── Method validation ─────────────────────────────────────────────────

#[tokio::test]
async fn register_rejects_get() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    // GET to a POST-only route → 405 Method Not Allowed
    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/auth/register")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn login_rejects_get() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    // GET to a POST-only route → 405 Method Not Allowed
    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn delete_rejects_post() {
    ensure_jwt_secret();
    let router = app(test_state_with_db().await);

    // POST to a DELETE-only route on a protected route → 401 Unauthorized
    // Middleware runs before method routing, so the missing auth header
    // causes require_auth to reject it before axum checks the HTTP method.
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/delete")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
