use crate::AppState;
use crate::handlers::errors::DownloadError;
use aws_sdk_s3::error::ProvideErrorMetadata;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderName, StatusCode, header};
use axum::response::IntoResponse;
use futures::StreamExt;
use tokio_util::io::ReaderStream;

/// Parses a string into a [`HeaderValue`](axum::http::HeaderValue), mapping
/// failures to [`DownloadError::HeaderInvalid`].
fn header_value(value: &str) -> Result<axum::http::HeaderValue, DownloadError> {
    value
        .parse()
        .map_err(|_| DownloadError::HeaderInvalid(value.to_string()))
}

pub async fn download(
    state: axum::extract::State<AppState>,
    Path(file_id): Path<String>,
) -> Result<impl IntoResponse, DownloadError> {
    let key = format!("uploads/{}", file_id);

    tracing::info!(bucket = %state.bucket, key = %key, "download request");

    // 1. Get the object stream from R2
    let resp = state
        .s3
        .get_object()
        .bucket(&state.bucket)
        .key(&key)
        .send()
        .await
        .map_err(|err| {
            let code = err.code().unwrap_or("Unknown");
            tracing::error!(
                bucket = %state.bucket,
                key = %key,
                error = %err,
                error_code = code,
                "failed to fetch object from R2"
            );
            match code {
                "NoSuchKey" | "404" => DownloadError::NotFound,
                "SlowDown" | "503" | "ServiceUnavailable" | "InternalError" | "500" => {
                    DownloadError::ServiceUnavailable
                }
                _ => DownloadError::FetchFailed,
            }
        })?;

    // 2. Extract metadata from the S3 response
    let content_type: Option<String> = resp.content_type().map(String::from);
    let content_length: Option<i64> = resp.content_length();
    let chunk_size: Option<u64> = resp
        .metadata()
        .and_then(|m| m.get("chunk-size"))
        .and_then(|v| v.parse().ok());

    // 3. Burn after reading: delete the object now that we have the stream.
    //    The data is already in transit from R2, so the stream continues to work.
    //    A failed delete is logged but does not fail the response — the cleanup
    //    worker will eventually remove orphaned objects.
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
    let body = Body::from_stream(stream.map(|result| result.map_err(std::io::Error::other)));

    // 5. Build response with metadata in headers
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        header_value("no-store, no-cache, must-revalidate")?,
    );
    headers.insert(header::PRAGMA, header_value("no-cache")?);
    headers.insert(header::EXPIRES, header_value("0")?);
    headers.insert(
        header::CONTENT_TYPE,
        header_value("application/octet-stream")?,
    );

    if let Some(ref ct) = content_type {
        headers.insert(HeaderName::from_static("x-content-type"), header_value(ct)?);
    }

    if let Some(cl) = content_length {
        headers.insert(header::CONTENT_LENGTH, header_value(&cl.to_string())?);
    }

    if let Some(cs) = chunk_size {
        headers.insert(
            HeaderName::from_static("x-chunk-size"),
            header_value(&cs.to_string())?,
        );
    }

    Ok((StatusCode::OK, headers, body).into_response())
}
