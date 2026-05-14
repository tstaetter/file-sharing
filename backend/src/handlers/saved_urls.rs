use crate::db::saved_url::SavedUrl;
use crate::handlers::errors::SavedUrlError;
use crate::middleware::AuthUser;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

// ── Constants ────────────────────────────────────────────────────────────

const DEFAULT_PER_PAGE: u64 = 10;
const MAX_PER_PAGE: u64 = 100;

// ── Request / Response types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SaveUrlRequest {
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListUrlsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    DEFAULT_PER_PAGE
}

#[derive(Debug, Serialize)]
pub struct SaveUrlResponse {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListUrlsResponse {
    pub urls: Vec<SaveUrlResponse>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

impl From<&SavedUrl> for SaveUrlResponse {
    fn from(s: &SavedUrl) -> Self {
        SaveUrlResponse {
            id: s.id.to_string(),
            url: s.url.clone(),
            title: s.title.clone(),
            created_at: s.created_at.to_rfc3339(),
        }
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────

/// Save a capability URL to the authenticated user's collection.
///
/// Requires a valid `Authorization: Bearer <token>` header. The user is
/// identified by the email in the token's `sub` claim. Returns the saved
/// URL record with its generated ID and timestamp.
pub async fn save_url(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<SaveUrlRequest>,
) -> Result<Json<SaveUrlResponse>, SavedUrlError> {
    // Validate required fields
    if req.url.trim().is_empty() {
        return Err(SavedUrlError::EmptyUrl);
    }

    // Get database handle
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| SavedUrlError::Database("no database configured".into()))?;
    let collection = db.collection::<SavedUrl>("saved_urls");

    // Create and insert the saved URL
    let saved = SavedUrl::new(auth_user.claims.sub, req.url, req.title);
    collection
        .insert_one(&saved)
        .await
        .map_err(|e| SavedUrlError::Database(e.to_string()))?;

    Ok(Json(SaveUrlResponse::from(&saved)))
}

/// List the authenticated user's saved URLs with pagination.
///
/// Requires a valid `Authorization: Bearer <token>` header. Returns URLs in
/// reverse chronological order (newest first). Supports `page` (default 1)
/// and `per_page` (default 10, max 100) query parameters.
pub async fn list_urls(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ListUrlsQuery>,
) -> Result<Json<ListUrlsResponse>, SavedUrlError> {
    // Validate pagination parameters
    if query.page < 1 {
        return Err(SavedUrlError::InvalidPage);
    }
    if query.per_page < 1 || query.per_page > MAX_PER_PAGE {
        return Err(SavedUrlError::InvalidPerPage);
    }

    // Get database handle
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| SavedUrlError::Database("no database configured".into()))?;
    let collection = db.collection::<SavedUrl>("saved_urls");

    // Count total documents for this user
    let filter = doc! { "user_email": &auth_user.claims.sub };
    let total = collection
        .count_documents(filter.clone())
        .await
        .map_err(|e| SavedUrlError::Database(e.to_string()))?;

    // Fetch the requested page, ordered by created_at descending
    let skip = (query.page - 1) * query.per_page;
    let mut cursor = collection
        .find(filter.clone())
        .sort(doc! { "created_at": -1 })
        .skip(skip)
        .limit(query.per_page as i64)
        .await
        .map_err(|e| SavedUrlError::Database(e.to_string()))?;

    // Collect results
    let mut urls = Vec::new();
    use futures::StreamExt;
    while let Some(result) = cursor.next().await {
        let doc = result.map_err(|e| SavedUrlError::Database(e.to_string()))?;
        urls.push(SaveUrlResponse::from(&doc));
    }

    Ok(Json(ListUrlsResponse {
        urls,
        page: query.page,
        per_page: query.per_page,
        total,
    }))
}

/// Delete a saved URL by its ID.
///
/// Requires a valid Authorization: Bearer token header. Only the owner
/// of the URL can delete it (enforced by matching user_email with the
/// authenticated users email). Returns 204 No Content on success,
/// 404 Not Found if the URL doesnt exist or belongs to another user.
pub async fn delete_url(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, SavedUrlError> {
    let db = state
        .database
        .as_ref()
        .ok_or_else(|| SavedUrlError::Database("no database configured".into()))?;
    let collection = db.collection::<SavedUrl>("saved_urls");
    let object_id = mongodb::bson::Uuid::parse_str(&id).map_err(|_| SavedUrlError::NotFound)?;
    // Delete only if the URL belongs to the authenticated user
    let filter = doc! {
        "_id": &object_id,
        "user_email": &auth_user.claims.sub,
    };

    let result = collection
        .delete_one(filter)
        .await
        .map_err(|e| SavedUrlError::Database(e.to_string()))?;

    if result.deleted_count == 0 {
        return Err(SavedUrlError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use chrono::Utc;

    // ── Unit tests: SaveUrlRequest deserialization ────────────────────

    #[test]
    fn test_save_url_request_deserializes() {
        let json = r#"{"url":"https://filez.zone/f/abc#key"}"#;
        let req: SaveUrlRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://filez.zone/f/abc#key");
        assert!(req.title.is_none());
    }

    #[test]
    fn test_save_url_request_with_title() {
        let json = r#"{"url":"https://filez.zone/f/abc#key","title":"My file"}"#;
        let req: SaveUrlRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.title.as_deref(), Some("My file"));
    }

    #[test]
    fn test_save_url_request_rejects_missing_url() {
        let json = r#"{"title":"test"}"#;
        serde_json::from_str::<SaveUrlRequest>(json).unwrap_err();
    }

    #[test]
    fn test_save_url_request_rejects_empty_object() {
        let json = r#"{}"#;
        serde_json::from_str::<SaveUrlRequest>(json).unwrap_err();
    }

    // ── Unit tests: ListUrlsQuery deserialization ─────────────────────

    #[test]
    fn test_list_urls_query_defaults() {
        let json = r#"{}"#;
        let query: ListUrlsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, DEFAULT_PER_PAGE);
    }

    #[test]
    fn test_list_urls_query_with_page_and_per_page() {
        let json = r#"{"page":3,"per_page":25}"#;
        let query: ListUrlsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, 3);
        assert_eq!(query.per_page, 25);
    }

    #[test]
    fn test_list_urls_query_allows_page_zero_for_validation_testing() {
        // Page 0 should be allowed during deserialization — the handler
        // will reject it with InvalidPage. This ensures the validation
        // logic is in the handler, not serde.
        let json = r#"{"page":0}"#;
        let query: ListUrlsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, 0);
    }

    // ── Unit tests: SaveUrlResponse serialization ─────────────────────

    #[test]
    fn test_save_url_response_serializes() {
        let resp = SaveUrlResponse {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            url: "https://filez.zone/f/abc#key".to_string(),
            title: Some("Vacation photos".to_string()),
            created_at: "2025-07-16T12:00:00+00:00".to_string(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["id"], "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(json["url"], "https://filez.zone/f/abc#key");
        assert_eq!(json["title"], "Vacation photos");
        assert_eq!(json["created_at"], "2025-07-16T12:00:00+00:00");
    }

    #[test]
    fn test_save_url_response_without_title() {
        let resp = SaveUrlResponse {
            id: "id123".to_string(),
            url: "https://example.com".to_string(),
            title: None,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["title"], serde_json::Value::Null);
    }

    // ── Unit tests: ListUrlsResponse serialization ────────────────────

    #[test]
    fn test_list_urls_response_serializes_empty() {
        let resp = ListUrlsResponse {
            urls: vec![],
            page: 1,
            per_page: 10,
            total: 0,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["urls"].as_array().unwrap().len(), 0);
        assert_eq!(json["page"], 1);
        assert_eq!(json["per_page"], 10);
        assert_eq!(json["total"], 0);
    }

    #[test]
    fn test_list_urls_response_serializes_with_items() {
        let resp = ListUrlsResponse {
            urls: vec![SaveUrlResponse {
                id: "id1".to_string(),
                url: "https://example.com/f/a#key".to_string(),
                title: Some("Test".to_string()),
                created_at: "2025-07-16T12:00:00+00:00".to_string(),
            }],
            page: 1,
            per_page: 10,
            total: 1,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["urls"][0]["id"], "id1");
        assert_eq!(json["total"], 1);
    }

    // ── Unit tests: Error types ───────────────────────────────────────

    #[test]
    fn test_saved_url_error_display() {
        assert_eq!(SavedUrlError::EmptyUrl.to_string(), "url cannot be empty");
        assert_eq!(
            SavedUrlError::InvalidToken("bad".into()).to_string(),
            "invalid or expired token: bad"
        );
        assert_eq!(
            SavedUrlError::InvalidPage.to_string(),
            "page must be at least 1"
        );
        assert!(SavedUrlError::InvalidPerPage
            .to_string()
            .contains("per_page must be between 1 and"));
    }

    #[test]
    fn test_saved_url_error_http_status() {
        let tests: Vec<(SavedUrlError, StatusCode)> = vec![
            (
                SavedUrlError::InvalidToken("x".into()),
                StatusCode::UNAUTHORIZED,
            ),
            (SavedUrlError::EmptyUrl, StatusCode::BAD_REQUEST),
            (
                SavedUrlError::Database("err".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (SavedUrlError::InvalidPage, StatusCode::BAD_REQUEST),
            (SavedUrlError::InvalidPerPage, StatusCode::BAD_REQUEST),
        ];

        for (error, expected_status) in tests {
            let response = error.into_response();
            assert_eq!(response.status(), expected_status);
        }
    }

    #[test]
    fn test_all_saved_url_errors_are_client_or_server_errors() {
        let errors = vec![
            SavedUrlError::InvalidToken("x".into()),
            SavedUrlError::EmptyUrl,
            SavedUrlError::Database("x".into()),
            SavedUrlError::InvalidPage,
            SavedUrlError::InvalidPerPage,
        ];

        for error in errors {
            let resp = error.into_response();
            assert!(
                resp.status().is_client_error() || resp.status().is_server_error(),
                "Expected 4xx/5xx, got {}",
                resp.status()
            );
        }
    }

    // ── Unit tests: Pagination bounds ─────────────────────────────────

    #[test]
    fn test_default_per_page_is_10() {
        assert_eq!(DEFAULT_PER_PAGE, 10);
    }

    #[test]
    fn test_max_per_page_is_100() {
        assert_eq!(MAX_PER_PAGE, 100);
    }

    #[test]
    fn test_query_default_page_is_1() {
        let query = ListUrlsQuery {
            page: default_page(),
            per_page: default_per_page(),
        };
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 10);
    }

    // ── Unit tests: Token integration ─────────────────────────────────

    // ── Unit tests: Response ID is a valid UUID string ────────────────

    #[test]
    fn test_save_url_response_id_is_string() {
        let resp = SaveUrlResponse {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            url: "url".to_string(),
            title: None,
            created_at: "ts".to_string(),
        };
        // UUID strings are 36 characters
        assert_eq!(resp.id.len(), 36);
    }

    // ── Unit tests: From<SavedUrl> conversion ─────────────────────────

    #[test]
    fn test_save_url_response_from_saved_url() {
        use chrono::TimeZone;
        let saved = SavedUrl {
            id: mongodb::bson::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            user_email: "user@test.com".to_string(),
            url: "https://filez.zone/f/abc#key".to_string(),
            title: Some("My file".to_string()),
            created_at: Utc.with_ymd_and_hms(2025, 7, 16, 12, 0, 0).unwrap(),
        };

        let response = SaveUrlResponse::from(&saved);
        assert_eq!(response.id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(response.url, "https://filez.zone/f/abc#key");
        assert_eq!(response.title, Some("My file".to_string()));
        assert!(response.created_at.starts_with("2025-07-16T12:00:00"));
    }

    #[test]
    fn test_save_url_response_excludes_user_email() {
        let saved = SavedUrl {
            id: mongodb::bson::Uuid::new(),
            user_email: "secret@test.com".to_string(),
            url: "url".to_string(),
            title: None,
            created_at: Utc::now(),
        };
        let response = SaveUrlResponse::from(&saved);
        let json = serde_json::to_value(&response).unwrap();
        // The response should NOT leak the user_email
        assert!(json.get("user_email").is_none());
    }
}
