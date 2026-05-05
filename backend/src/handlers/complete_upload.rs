use crate::AppState;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use axum::Json;
use axum::extract::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CompleteRequest {
    key: String,
    upload_id: String,
    parts: Vec<PartETag>,
}

#[derive(Deserialize)]
struct PartETag {
    part_number: i32,
    etag: String,
}

pub async fn complete_upload(State(state): State<AppState>, Json(req): Json<CompleteRequest>) {
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
        .unwrap();
}
