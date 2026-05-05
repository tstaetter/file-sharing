use crate::AppState;
use axum::Json;
use axum::extract::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AbortRequest {
    key: String,
    upload_id: String,
}

pub async fn abort_upload(State(state): State<AppState>, Json(req): Json<AbortRequest>) {
    state
        .s3
        .abort_multipart_upload()
        .bucket(&state.bucket)
        .key(&req.key)
        .upload_id(&req.upload_id)
        .send()
        .await
        .unwrap();
}
