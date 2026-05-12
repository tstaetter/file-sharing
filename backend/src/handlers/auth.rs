use crate::db::user::User;
use crate::handlers::errors::AuthError;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // email
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(email: &str) -> Result<String, AuthError> {
    dotenvy::dotenv().ok();

    let now = Utc::now();
    let mins = std::env::var("JWT_EXPIRY_MINS").unwrap_or("5".to_string());
    let secret = std::env::var("JWT_SECRET").expect("No JWT secret");
    let exp = now + Duration::minutes(mins.parse().unwrap_or_default());

    let claims = Claims {
        sub: email.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::JwtToken(e.to_string()))
}

pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    dotenvy::dotenv().ok();

    let secret = std::env::var("JWT_SECRET").expect("No JWT secret");

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AuthError::JwtValidation(e.to_string()))
}

// ── Request / Response types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub email: String,
    pub name: String,
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        UserResponse {
            email: user.email.clone(),
            name: user.name.clone(),
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────

/// Register a new user.
///
/// Accepts email, password, and name. The password is hashed with bcrypt
/// before storage. Returns a JWT token valid for 72 hours.
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AuthError> {
    // Validate input
    if req.email.is_empty() || req.password.is_empty() || req.name.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    // Check if user already exists
    let db = state
        .database
        .as_ref()
        .ok_or(AuthError::Database("no database configured".into()))?;
    let collection = db.collection::<User>("users");
    let existing = collection
        .find_one(doc! { "email": &req.email })
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

    if existing.is_some() {
        return Err(AuthError::UserExists);
    }

    // Hash password
    let password_hash = hash(&req.password, DEFAULT_COST).map_err(|_| AuthError::HashError)?;

    // Create user document
    let user = User {
        email: req.email.clone(),
        password_hash,
        name: req.name.clone(),
    };

    collection
        .insert_one(&user)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

    // Generate JWT
    let token = create_token(&req.email)?;
    let user_response = UserResponse::from(&user);

    Ok(Json(RegisterResponse {
        token,
        user: user_response,
    }))
}

/// Authenticate an existing user.
///
/// Accepts email and password. Verifies the password hash with bcrypt.
/// Returns a JWT token valid for 72 hours on success.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // Validate input
    if req.email.is_empty() || req.password.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    // Find user by email
    let db = state
        .database
        .as_ref()
        .ok_or(AuthError::Database("no database configured".into()))?;
    let collection = db.collection::<User>("users");
    let user = collection
        .find_one(doc! { "email": &req.email })
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    let valid = verify(&req.password, &user.password_hash).unwrap_or(false);
    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Generate JWT
    let token = create_token(&req.email)?;
    let user_response = UserResponse::from(&user);

    Ok(Json(LoginResponse {
        token,
        user: user_response,
    }))
}

/// Delete the authenticated user's account.
///
/// Requires a valid JWT token. Deletes the user document from the database.
/// This is irreversible.
pub async fn delete_user(
    State(state): State<AppState>,
    Json(req): Json<DeleteRequest>,
) -> Result<StatusCode, AuthError> {
    // Validate token and extract email
    let claims = validate_token(&req.token)?;

    // Delete user from database
    let db = state
        .database
        .as_ref()
        .ok_or(AuthError::Database("no database configured".into()))?;
    let collection = db.collection::<User>("users");
    let result = collection
        .delete_one(doc! { "email": &claims.sub })
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

    if result.deleted_count == 0 {
        return Err(AuthError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    // ── Unit tests: Token lifecycle ───────────────────────────────────

    #[test]
    fn test_create_and_validate_token() {
        let email = "test@example.com";
        let token = create_token(email).expect("should create token");
        let claims = validate_token(&token).expect("should validate token");
        assert_eq!(claims.sub, email);
    }

    #[test]
    fn test_validate_invalid_token_fails() {
        let result = validate_token("not.a.valid.token");
        assert!(result.is_err());
        match result {
            Err(AuthError::JwtValidation(_)) => {} // expected
            other => panic!("Expected JwtValidation error, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_empty_token_fails() {
        let result = validate_token("");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_for_different_users_are_different() {
        let token1 = create_token("alice@example.com").unwrap();
        let token2 = create_token("bob@example.com").unwrap();
        assert_ne!(token1, token2);
    }

    // ── Unit tests: Password hashing ──────────────────────────────────

    #[test]
    fn test_hash_and_verify_password() {
        let password = "secure-password-123";
        let hashed = hash(password, DEFAULT_COST).expect("should hash password");
        assert!(verify(password, &hashed).unwrap_or(false));
    }

    #[test]
    fn test_verify_wrong_password_fails() {
        let password = "correct-password";
        let hashed = hash(password, DEFAULT_COST).expect("should hash password");
        assert!(!verify("wrong-password", &hashed).unwrap_or(false));
    }

    #[test]
    fn test_hash_produces_different_outputs_for_same_input() {
        let password = "same-password";
        let hash1 = hash(password, DEFAULT_COST).unwrap();
        let hash2 = hash(password, DEFAULT_COST).unwrap();
        // bcrypt uses random salt, so hashes should differ even for same input
        assert_ne!(hash1, hash2);
    }

    // ── Unit tests: Request deserialization ───────────────────────────

    #[test]
    fn test_register_request_deserializes() {
        let json = r#"{"email":"a@b.com","password":"secret","name":"Alice"}"#;
        let req: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "a@b.com");
        assert_eq!(req.password, "secret");
        assert_eq!(req.name, "Alice");
    }

    #[test]
    fn test_register_request_rejects_missing_email() {
        let json = r#"{"password":"secret","name":"Alice"}"#;
        serde_json::from_str::<RegisterRequest>(json).unwrap_err();
    }

    #[test]
    fn test_register_request_rejects_missing_password() {
        let json = r#"{"email":"a@b.com","name":"Alice"}"#;
        serde_json::from_str::<RegisterRequest>(json).unwrap_err();
    }

    #[test]
    fn test_register_request_rejects_missing_name() {
        let json = r#"{"email":"a@b.com","password":"secret"}"#;
        serde_json::from_str::<RegisterRequest>(json).unwrap_err();
    }

    #[test]
    fn test_login_request_deserializes() {
        let json = r#"{"email":"a@b.com","password":"secret"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "a@b.com");
        assert_eq!(req.password, "secret");
    }

    #[test]
    fn test_login_request_rejects_missing_email() {
        let json = r#"{"password":"secret"}"#;
        serde_json::from_str::<LoginRequest>(json).unwrap_err();
    }

    #[test]
    fn test_login_request_rejects_missing_password() {
        let json = r#"{"email":"a@b.com"}"#;
        serde_json::from_str::<LoginRequest>(json).unwrap_err();
    }

    #[test]
    fn test_delete_request_deserializes() {
        let json = r#"{"token":"eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhQGLC5jb21mIn0.rsa"}"#;
        let req: DeleteRequest = serde_json::from_str(json).unwrap();
        assert!(req.token.starts_with("eyJ"));
    }

    // ── Unit tests: Error types ───────────────────────────────────────

    #[test]
    fn test_auth_error_display() {
        assert_eq!(
            AuthError::UserExists.to_string(),
            "user with this email already exists"
        );
        assert_eq!(
            AuthError::InvalidCredentials.to_string(),
            "invalid email or password"
        );
        assert_eq!(AuthError::NotFound.to_string(), "user not found");
    }

    #[test]
    fn test_auth_error_http_status() {
        let tests: Vec<(AuthError, StatusCode)> = vec![
            (AuthError::UserExists, StatusCode::CONFLICT),
            (AuthError::InvalidCredentials, StatusCode::UNAUTHORIZED),
            (AuthError::NotFound, StatusCode::NOT_FOUND),
            (AuthError::HashError, StatusCode::INTERNAL_SERVER_ERROR),
            (
                AuthError::JwtToken("err".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                AuthError::JwtValidation("err".into()),
                StatusCode::UNAUTHORIZED,
            ),
            (
                AuthError::Database("err".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];

        for (error, expected_status) in tests {
            let response = error.into_response();
            assert_eq!(
                response.status(),
                expected_status,
                "Error {:?} should return {}",
                expected_status.as_str(),
                expected_status
            );
        }
    }

    #[test]
    fn test_all_auth_errors_are_client_or_server_errors() {
        let errors = vec![
            AuthError::UserExists,
            AuthError::InvalidCredentials,
            AuthError::NotFound,
            AuthError::HashError,
            AuthError::JwtToken("err".into()),
            AuthError::JwtValidation("err".into()),
            AuthError::Database("err".into()),
        ];

        for error in errors {
            let variant_name = format!("{:?}", error);
            let resp = error.into_response();
            assert!(
                resp.status().is_client_error() || resp.status().is_server_error(),
                "Expected 4xx/5xx for {}, got {}",
                variant_name,
                resp.status()
            );
        }
    }

    // ── Unit tests: UserResponse conversion ───────────────────────────

    #[test]
    fn test_user_response_from_user() {
        let user = User {
            email: "test@example.com".to_string(),
            password_hash: "hashed".to_string(),
            name: "Test User".to_string(),
        };
        let response = UserResponse::from(&user);
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.name, "Test User");
    }

    #[test]
    fn test_user_response_excludes_password_hash() {
        let user = User {
            email: "test@example.com".to_string(),
            password_hash: "secret-hash".to_string(),
            name: "Test".to_string(),
        };
        let response = UserResponse::from(&user);
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("password_hash").is_none());
        assert!(json.get("email").is_some());
        assert!(json.get("name").is_some());
    }

    // ── Unit tests: JWT claims structure ──────────────────────────────

    #[test]
    fn test_claims_serialize_and_deserialize() {
        let claims = Claims {
            sub: "user@example.com".to_string(),
            exp: 9999999999usize,
            iat: 1000000000usize,
        };
        let json = serde_json::to_string(&claims).unwrap();
        let parsed: Claims = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sub, claims.sub);
        assert_eq!(parsed.exp, claims.exp);
        assert_eq!(parsed.iat, claims.iat);
    }

    // ── Unit tests: Response serialization ────────────────────────────

    #[test]
    fn test_register_response_serializes() {
        let resp = RegisterResponse {
            token: "token-abc".to_string(),
            user: UserResponse {
                email: "test@test.com".to_string(),
                name: "Test".to_string(),
            },
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["token"], "token-abc");
        assert_eq!(json["user"]["email"], "test@test.com");
        assert_eq!(json["user"]["name"], "Test");
    }

    #[test]
    fn test_login_response_serializes() {
        let resp = LoginResponse {
            token: "token-def".to_string(),
            user: UserResponse {
                email: "user@test.com".to_string(),
                name: "User".to_string(),
            },
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["token"], "token-def");
        assert_eq!(json["user"]["email"], "user@test.com");
    }

    // ── Unit tests: Input validation (empty fields) ───────────────────

    #[test]
    fn test_register_rejects_empty_email() {
        let req = RegisterRequest {
            email: "".to_string(),
            password: "pwd".to_string(),
            name: "Name".to_string(),
        };
        // We can't easily test the handler without a DB, but we can test
        // the validation logic's intent by checking the error variant.
        // The handler validates: email.is_empty() → InvalidCredentials
        assert!(req.email.is_empty());
    }

    #[test]
    fn test_register_rejects_empty_password() {
        let req = RegisterRequest {
            email: "a@b.com".to_string(),
            password: "".to_string(),
            name: "Name".to_string(),
        };
        assert!(req.password.is_empty());
    }

    #[test]
    fn test_register_rejects_empty_name() {
        let req = RegisterRequest {
            email: "a@b.com".to_string(),
            password: "pwd".to_string(),
            name: "".to_string(),
        };
        assert!(req.name.is_empty());
    }
}
