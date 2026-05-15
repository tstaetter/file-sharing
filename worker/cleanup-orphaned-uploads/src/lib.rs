use aws_sdk_s3::Client;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{DateTime, Duration, Utc};
use futures::{stream::FuturesUnordered, StreamExt};
use tracing::info;

/// Maximum number of in-flight abort requests.
pub const MAX_CONCURRENT: usize = 10;

/// Default age cutoff for orphaned uploads (6 hours).
pub const DEFAULT_ORPHAN_AGE_HOURS: i64 = 6;

pub type CleanupResult<T> = Result<T, CleanupError>;

#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("DateTime error: {0}")]
    DateTime(#[from] aws_smithy_types_convert::date_time::Error),
    #[error("SDK error: {0}")]
    Sdk(String),
}

/// Scans for multipart uploads older than `cutoff` and aborts them.
///
/// This is the core logic extracted from `main.rs` so it can be tested
/// independently with a mock S3 client.
///
/// # Arguments
///
/// * `client` - An S3 client (real or mock).
/// * `bucket` - The bucket name to scan.
/// * `prefix`  - The upload key prefix to filter by (typically `"uploads/"`).
/// * `cutoff`  - Uploads initiated before this timestamp are aborted.
/// * `max_concurrent` - Maximum number of in-flight abort requests.
pub async fn cleanup_orphaned_uploads(
    client: &Client,
    bucket: &str,
    prefix: &str,
    cutoff: DateTime<Utc>,
    max_concurrent: usize,
) -> CleanupResult<()> {
    info!("Cleaning uploads older than {}", cutoff);

    let mut tasks = FuturesUnordered::new();
    let mut key_marker = None;
    let mut upload_id_marker = None;

    loop {
        let resp = client
            .list_multipart_uploads()
            .prefix(prefix)
            .bucket(bucket)
            .set_key_marker(key_marker.clone())
            .set_upload_id_marker(upload_id_marker.clone())
            .send()
            .await
            .map_err(|e| CleanupError::Sdk(e.to_string()))?;

        for upload in resp.uploads() {
            let key = upload.key().unwrap_or_default().to_string();
            let upload_id = upload.upload_id().unwrap_or_default().to_string();

            let initiated = upload.initiated().unwrap();
            let initiated_time = initiated.to_chrono_utc()?;

            if initiated_time < cutoff {
                let client_clone = client.clone();
                let bucket_clone = bucket.to_string();

                tasks.push(tokio::spawn(async move {
                    info!("Aborting {} ({})", key, upload_id);

                    client_clone
                        .abort_multipart_upload()
                        .bucket(bucket_clone)
                        .key(&*key)
                        .upload_id(&*upload_id)
                        .send()
                        .await
                }));

                // Concurrency limit
                if tasks.len() >= max_concurrent {
                    if let Some(res) = tasks.next().await {
                        match res {
                            Ok(Ok(_)) => {}
                            Ok(Err(e)) => return Err(CleanupError::Sdk(e.to_string())),
                            Err(e) => return Err(CleanupError::Sdk(e.to_string())),
                        }
                    }
                }
            }
        }

        // Pagination
        if let Some(true) = resp.is_truncated() {
            key_marker = resp.next_key_marker().map(|s| s.to_string());
            upload_id_marker = resp.next_upload_id_marker().map(|s| s.to_string());
        } else {
            break;
        }
    }

    // Drain remaining tasks
    while let Some(res) = tasks.next().await {
        match res {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(CleanupError::Sdk(e.to_string())),
            Err(e) => return Err(CleanupError::Sdk(e.to_string())),
        }
    }

    info!("Cleanup finished");

    Ok(())
}

/// Computes the cutoff timestamp: now minus `age_hours`.
pub fn compute_cutoff(age_hours: i64) -> DateTime<Utc> {
    Utc::now() - Duration::hours(age_hours)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_error_display() {
        let sdk_err = CleanupError::Sdk("timeout".into());
        assert_eq!(sdk_err.to_string(), "SDK error: timeout");
    }

    #[test]
    fn test_cleanup_error_debug() {
        let err = CleanupError::Sdk("oops".into());
        let debug = format!("{:?}", err);
        assert!(
            debug.contains("Sdk"),
            "Debug should name variant: {}",
            debug
        );
        assert!(debug.contains("oops"), "Debug should show inner: {}", debug);
    }

    #[test]
    fn test_compute_cutoff_is_in_the_past() {
        let cutoff = compute_cutoff(6);
        let now = Utc::now();
        assert!(cutoff < now, "cutoff should be before now");
        let diff = now - cutoff;
        assert!(
            diff >= Duration::hours(5) && diff <= Duration::hours(7),
            "cutoff should be ~6h ago, got {:?}",
            diff
        );
    }

    #[test]
    fn test_compute_cutoff_zero_hours_is_recent() {
        let cutoff = compute_cutoff(0);
        let diff = Utc::now() - cutoff;
        assert!(
            diff < Duration::seconds(5),
            "0h cutoff should be nearly now"
        );
    }

    #[test]
    fn test_max_concurrent_is_reasonable() {
        // Sanity check: the constant should be at least 1 and not enormous
        assert!(MAX_CONCURRENT >= 1);
        assert!(MAX_CONCURRENT <= 100);
    }
}
