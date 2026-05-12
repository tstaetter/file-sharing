use crate::handlers::errors::AbortUploadError;
use crate::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AbortRequest {
    pub key: String,
    pub upload_id: String,
}

pub async fn abort_upload(
    State(state): State<AppState>,
    Json(req): Json<AbortRequest>,
) -> Result<impl IntoResponse, AbortUploadError> {
    state
        .s3
        .abort_multipart_upload()
        .bucket(&state.bucket)
        .key(&req.key)
        .upload_id(&req.upload_id)
        .send()
        .await
        .map_err(|e| AbortUploadError::Sdk(e.to_string()))?;

    Ok(())
}
