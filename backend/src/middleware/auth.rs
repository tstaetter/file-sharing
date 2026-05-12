use crate::handlers::{validate_token, Claims};
use axum::extract::FromRequestParts;
use axum::extract::Request;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Authenticated user extracted from a validated JWT.
///
/// Set as a request extension by [`require_auth`] middleware. Handlers on
/// protected routes extract it to access verified claims without re-validating
/// the token.
///
/// # Example
///
/// ```rust,ignore
/// async fn my_handler(auth_user: AuthUser) -> Json<MyResponse> {
///     let email = &auth_user.claims.sub;
///     // ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub claims: Claims,
}

/// Errors returned when [`AuthUser`] extraction fails.
#[derive(Debug, thiserror::Error)]
pub enum AuthMiddlewareError {
    #[error("authentication required")]
    MissingAuth,
}

impl IntoResponse for AuthMiddlewareError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
    }
}

/// Extract [`AuthUser`] from request extensions.
///
/// This extractor pulls the [`AuthUser`] that was inserted by the
/// [`require_auth`] middleware. If the extension is missing (i.e. the request
/// was not authenticated by the middleware), it returns
/// [`AuthMiddlewareError::MissingAuth`] which maps to `401 Unauthorized`.
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AuthMiddlewareError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AuthMiddlewareError::MissingAuth)
    }
}

/// Middleware that validates the `Authorization: Bearer <token>` header.
///
/// On success, the validated claims are stored as an [`AuthUser`] extension
/// so downstream handlers can access them without re-validating the token.
///
/// On failure, the request is rejected with `401 Unauthorized`.
pub async fn require_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..]; // Skip "Bearer "
            match validate_token(token) {
                Ok(claims) => {
                    let mut request = request;
                    request.extensions_mut().insert(AuthUser { claims });
                    Ok(next.run(request).await)
                }
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    // ── AuthMiddlewareError tests ────────────────────────────────────

    #[test]
    fn test_auth_middleware_error_display() {
        assert_eq!(
            AuthMiddlewareError::MissingAuth.to_string(),
            "authentication required"
        );
    }

    #[test]
    fn test_auth_middleware_error_http_status() {
        let response = AuthMiddlewareError::MissingAuth.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_auth_middleware_error_is_client_error() {
        let response = AuthMiddlewareError::MissingAuth.into_response();
        assert!(response.status().is_client_error());
    }

    // ── AuthUser tests ───────────────────────────────────────────────

    #[test]
    fn test_auth_user_clone() {
        let claims = Claims {
            sub: "user@example.com".to_string(),
            exp: 1234567890,
            iat: 1234560000,
        };
        let auth_user = AuthUser { claims };
        let cloned = auth_user.clone();
        assert_eq!(cloned.claims.sub, "user@example.com");
        assert_eq!(cloned.claims.exp, 1234567890);
        assert_eq!(cloned.claims.iat, 1234560000);
    }

    #[test]
    fn test_auth_user_debug() {
        let claims = Claims {
            sub: "user@example.com".to_string(),
            exp: 1234567890,
            iat: 1234560000,
        };
        let auth_user = AuthUser { claims };
        let debug = format!("{:?}", auth_user);
        assert!(debug.contains("AuthUser"));
        assert!(debug.contains("user@example.com"));
    }

    // ── Bearer token parsing logic ───────────────────────────────────

    #[test]
    fn test_bearer_prefix_is_seven_chars() {
        // "Bearer " is 7 characters — the slice &header[7..] skips it
        assert_eq!("Bearer ".len(), 7);
    }

    #[test]
    fn test_bearer_token_extraction() {
        let header = "Bearer eyJhbGciOiJIUzI1NiJ9";
        assert!(header.starts_with("Bearer "));
        let token = &header[7..];
        assert_eq!(token, "eyJhbGciOiJIUzI1NiJ9");
    }

    #[test]
    fn test_non_bearer_header_rejected() {
        let header = "Basic dXNlcjpwYXNz";
        assert!(!header.starts_with("Bearer "));
    }

    #[test]
    fn test_empty_authorization_header_rejected() {
        let header = "";
        assert!(!header.starts_with("Bearer "));
    }

    #[test]
    fn test_bearer_without_space_rejected() {
        // "Bearer" without trailing space should not match
        let header = "Bearer";
        assert!(!header.starts_with("Bearer "));
    }
}
