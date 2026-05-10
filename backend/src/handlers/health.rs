use axum::{http::StatusCode, response::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

/// Health check endpoint for Koyeb and other orchestrators.
/// Koyeb uses this to determine if the service is ready.
/// Returns 200 OK with `{"status":"ok"}`.
pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}
