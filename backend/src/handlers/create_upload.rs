use crate::AppState;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateRequest {
    file_id: String,
    content_type: Option<String>,
}

#[derive(Serialize)]
pub struct CreateResponse {
    upload_id: String,
    key: String,
}

pub async fn create_upload(
    State(state): State<AppState>,
    Json(req): Json<CreateRequest>,
) -> Json<CreateResponse> {
    let key = format!("uploads/{}", req.file_id);

    let mut builder = state
        .s3
        .create_multipart_upload()
        .bucket(&state.bucket)
        .key(&key);

    if let Some(ct) = req.content_type {
        builder = builder.content_type(ct);
    }

    let resp = builder.send().await.unwrap();

    Json(CreateResponse {
        upload_id: resp.upload_id().unwrap().to_string(),
        key,
    })
}
