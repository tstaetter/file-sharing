use crate::AppState;
use aws_sdk_s3::error::ProvideErrorMetadata;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderName, StatusCode, header};
use axum::response::IntoResponse;
use futures::StreamExt;
use tokio_util::io::ReaderStream;

pub async fn download(
    state: axum::extract::State<AppState>,
    Path(file_id): Path<String>,
) -> impl IntoResponse {
    let key = format!("uploads/{}", file_id);

    tracing::info!(bucket = %state.bucket, key = %key, "download request");

    // 1. Get the object stream from R2
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
                error_code = ?err.code(),
                "failed to fetch object from R2"
            );
            return (StatusCode::NOT_FOUND, "file not found").into_response();
        }
    };

    // 2. Extract metadata from the S3 response
    let content_type: Option<String> = resp.content_type().map(String::from);
    let chunk_size: Option<u64> = resp
        .metadata()
        .and_then(|m| m.get("chunk-size"))
        .and_then(|v| v.parse().ok());

    // 3. Burn after reading: delete the object now that we have the stream.
    //    The data is already in transit from R2, so the stream continues to work.
    match state
        .s3
        .delete_object()
        .bucket(&state.bucket)
        .key(&key)
        .send()
        .await
    {
        Ok(_) => {
            tracing::info!(key = %key, "object deleted from R2 (burn after reading)");
        }
        Err(err) => {
            tracing::warn!(key = %key, error = %err, "failed to delete object from R2");
        }
    }

    // 4. Convert the streaming body to an Axum response
    let body_reader = resp.body.into_async_read();
    let stream = ReaderStream::new(body_reader);
    let body = Body::from_stream(
        stream.map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))),
    );

    // 5. Build response with metadata in headers
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        "no-store, no-cache, must-revalidate".parse().unwrap(),
    );
    headers.insert(header::PRAGMA, "no-cache".parse().unwrap());
    headers.insert(header::EXPIRES, "0".parse().unwrap());
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );

    if let Some(ref ct) = content_type {
        headers.insert(
            HeaderName::from_static("x-content-type"),
            ct.parse().unwrap(),
        );
    }

    if let Some(cs) = chunk_size {
        headers.insert(
            HeaderName::from_static("x-chunk-size"),
            cs.to_string().parse().unwrap(),
        );
    }

    (StatusCode::OK, headers, body).into_response()
}
