use crate::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateRequest {
    pub file_id: String,
    pub content_type: Option<String>,
    pub chunk_size: Option<u64>,
}

#[derive(Serialize)]
pub struct CreateResponse {
    pub upload_id: String,
    pub key: String,
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

    if let Some(cs) = req.chunk_size {
        builder = builder.metadata("chunk-size", cs.to_string());
    }

    let resp = builder.send().await.unwrap();

    Json(CreateResponse {
        upload_id: resp.upload_id().unwrap_or_default().to_string(),
        key,
    })
}
