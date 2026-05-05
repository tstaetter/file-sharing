use crate::AppState;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub struct SignPartsRequest {
    key: String,
    upload_id: String,
    part_numbers: Vec<i32>,
}

#[derive(Serialize)]
pub struct SignedPart {
    part_number: i32,
    url: String,
}

pub async fn sign_parts(
    State(state): State<AppState>,
    Json(req): Json<SignPartsRequest>,
) -> Json<Vec<SignedPart>> {
    let mut urls = Vec::new();

    for part_number in req.part_numbers {
        let presigned = state
            .s3
            .upload_part()
            .bucket(&state.bucket)
            .key(&req.key)
            .upload_id(&req.upload_id)
            .part_number(part_number)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(3600)).unwrap())
            .await
            .unwrap();

        urls.push(SignedPart {
            part_number,
            url: presigned.uri().to_string(),
        });
    }

    Json(urls)
}
