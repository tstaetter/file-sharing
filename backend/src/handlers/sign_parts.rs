use crate::{handlers::errors::SignPartsError, AppState};
use aws_sdk_s3::presigning::PresigningConfig;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct SignPartsRequest {
    pub key: String,
    pub upload_id: String,
    pub part_numbers: Vec<i32>,
}

#[derive(Serialize)]
pub struct SignedPart {
    pub part_number: i32,
    pub url: String,
}

pub async fn sign_parts(
    State(state): State<AppState>,
    Json(req): Json<SignPartsRequest>,
) -> Result<Json<Vec<SignedPart>>, SignPartsError> {
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
            .map_err(|e| SignPartsError::Sdk(e.to_string()))?;

        urls.push(SignedPart {
            part_number,
            url: presigned.uri().to_string(),
        });
    }

    Ok(Json(urls))
}
