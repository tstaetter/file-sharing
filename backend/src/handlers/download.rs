use crate::AppState;
use axum::Json;
use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::Serialize;

#[derive(Clone, Default, Serialize)]
pub struct StoredFile {
    /// Base64-encoded ciphertext (concatenated per-chunk IV || ciphertext blocks).
    pub data: String,
    /// Base64-encoded AES-GCM nonce used for the first chunk (legacy; per-chunk IVs
    /// are embedded in `data`).
    pub nonce: String,
    /// The original file's MIME type (e.g. "image/png"), stored as S3 metadata
    /// during upload. Used by the frontend to infer the correct file extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

pub async fn download(
    state: axum::extract::State<AppState>,
    Path(file_id): Path<String>,
) -> impl IntoResponse {
    let key = format!("uploads/{}", file_id);

    tracing::info!(
        bucket = %state.bucket,
        key = %key,
        "download request"
    );

    let resp = match state
        .s3
        .get_object()
        .bucket(&state.bucket)
        .key(&key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(err) => {
            tracing::error!(
                bucket = %state.bucket,
                key = %key,
                error = %err,
                "failed to fetch object from R2"
            );
            return (StatusCode::NOT_FOUND, "file not found").into_response();
        }
    };

    // Extract the original MIME type from object metadata (set during upload).
    let content_type: Option<String> = resp.content_type().map(String::from);

    let data = match resp.body.collect().await {
        Ok(bytes) => bytes.into_bytes().to_vec(),
        Err(err) => {
            tracing::error!(
                bucket = %state.bucket,
                key = %key,
                error = %err,
                "failed to read object body from R2"
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to read file").into_response();
        }
    };

    let content_length = data.len();

    // Burn after reading — delete the object from R2 now that we have it in memory.
    // If the delete fails, warn but still serve the data to the client.
    match state
        .s3
        .delete_object()
        .bucket(&state.bucket)
        .key(&key)
        .send()
        .await
    {
        Ok(_) => {
            tracing::info!(
                bucket = %state.bucket,
                key = %key,
                bytes = content_length,
                content_type = ?content_type,
                "download complete — object deleted from R2"
            );
        }
        Err(err) => {
            tracing::warn!(
                bucket = %state.bucket,
                key = %key,
                bytes = content_length,
                content_type = ?content_type,
                error = %err,
                "download complete — but failed to delete object from R2"
            );
        }
    }

    // Encode as base64 for the JSON response.
    let data_b64 = BASE64.encode(&data);
    // The per-chunk encryption model means the "nonce" is per-chunk and embedded
    // in the data stream. We return a zero nonce here; the client decodes each
    // chunk's IV from the concatenated `IV || ciphertext` blocks.
    let nonce_b64 = BASE64.encode([0u8; 12]);

    let stored = StoredFile {
        data: data_b64,
        nonce: nonce_b64,
        content_type,
    };

    (
        StatusCode::OK,
        [
            (header::CACHE_CONTROL, "no-store, no-cache, must-revalidate"),
            (header::PRAGMA, "no-cache"),
            (header::EXPIRES, "0"),
        ],
        Json(stored),
    )
        .into_response()
}
