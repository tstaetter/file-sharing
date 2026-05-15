use crate::{handlers::errors::CreateUploadError, AppState, CreateRequest, CreateResponse};
use axum::{extract::State, Json};

pub async fn create_upload(
    State(state): State<AppState>,
    Json(req): Json<CreateRequest>,
) -> Result<Json<CreateResponse>, CreateUploadError> {
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

    let resp = builder
        .send()
        .await
        .map_err(|e| CreateUploadError::ResponseBuilder(e.to_string()))?;

    Ok(Json(CreateResponse {
        upload_id: resp.upload_id().unwrap_or_default().to_string(),
        key,
    }))
}
