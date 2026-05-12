use crate::handlers::errors::CompleteUploadError;
use crate::AppState;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CompleteRequest {
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<PartETag>,
}

#[derive(Debug, Deserialize)]
pub struct PartETag {
    pub part_number: i32,
    pub etag: String,
}

pub async fn complete_upload(
    State(state): State<AppState>,
    Json(req): Json<CompleteRequest>,
) -> Result<impl IntoResponse, CompleteUploadError> {
    let completed_parts = req
        .parts
        .into_iter()
        .map(|p| {
            CompletedPart::builder()
                .set_part_number(Some(p.part_number))
                .set_e_tag(Some(p.etag))
                .build()
        })
        .collect();

    let upload = CompletedMultipartUpload::builder()
        .set_parts(Some(completed_parts))
        .build();

    state
        .s3
        .complete_multipart_upload()
        .bucket(&state.bucket)
        .key(&req.key)
        .upload_id(&req.upload_id)
        .multipart_upload(upload)
        .send()
        .await
        .map_err(|e| CompleteUploadError::Sdk(e.to_string()))?;

    Ok(())
}
