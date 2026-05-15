// Integration tests for the cleanup_orphaned_uploads worker.
//
// These tests use `StaticReplayClient` from `aws-smithy-runtime` to
// mock the S3 API at the HTTP level, so no real R2 bucket is needed.

use aws_sdk_s3::Client;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::date_time::DateTime;
use aws_smithy_types::date_time::Format;
use chrono::{Duration, Utc};
use cleanup_orphaned_uploads::{cleanup_orphaned_uploads, MAX_CONCURRENT};

// ── Helpers ──────────────────────────────────────────────────────────────

/// Builds an S3 XML response body for `ListMultipartUploads`.
fn list_response_xml(
    uploads: &[(impl AsRef<str>, impl AsRef<str>, impl AsRef<str>)],
    truncated: bool,
) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <ListMultipartUploadsResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">\n\
         <Bucket>test-bucket</Bucket>\n\
         <KeyMarker></KeyMarker>\n\
         <UploadIdMarker></UploadIdMarker>\n\
         <MaxUploads>1000</MaxUploads>\n\
         <IsTruncated>",
    );
    xml.push_str(if truncated { "true" } else { "false" });
    xml.push_str("</IsTruncated>\n");

    for (key, upload_id, initiated) in uploads {
        xml.push_str(&format!(
            "  <Upload>\n    <Key>{}</Key>\n    <UploadId>{}</UploadId>\n    <Initiated>{}</Initiated>\n  </Upload>\n",
            key.as_ref(),
            upload_id.as_ref(),
            initiated.as_ref()
        ));
    }

    xml.push_str("</ListMultipartUploadsResult>\n");
    xml
}

/// Builds the list-multipart-uploads request URI with prefix.
fn list_uri(bucket: &str, prefix: &str) -> String {
    format!("/{}?uploads&prefix={}", bucket, percent_encode(prefix))
}

/// Percent-encodes a string for URL query parameters.
fn percent_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b'/' => result.push_str("%2F"),
            _ => result.push_str(&format!("%{:02X}", byte)),
        }
    }
    result
}

/// Creates an S3 client backed by a mock HTTP client.
fn mock_client(events: Vec<ReplayEvent>) -> Client {
    let http_client = StaticReplayClient::new(events);
    let config = aws_sdk_s3::Config::builder()
        .behavior_version(aws_config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url("https://test.r2.cloudflarestorage.com")
        .force_path_style(true)
        .http_client(http_client.clone())
        .build();
    Client::from_conf(config)
}

/// Creates a request → 204 response pair for an abort request.
fn abort_event(bucket: &str, key: &str, upload_id: &str) -> ReplayEvent {
    ReplayEvent::new(
        http::Request::builder()
            .uri(format!(
                "https://test.r2.cloudflarestorage.com/{}?x-id=DeleteObject&uploadId={}",
                percent_encode(&format!("{}/{}", bucket, key)),
                percent_encode(upload_id)
            ))
            .method("DELETE")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder()
            .status(204)
            .body(SdkBody::from(""))
            .unwrap(),
    )
}

// ── Tests ────────────────────────────────────────────────────────────────

/// No uploads → nothing to abort, success.
#[tokio::test]
async fn test_empty_upload_list() {
    let cutoff = Utc::now() - Duration::hours(6);

    let empty: &[(&str, &str, &str)] = &[];
    let events = vec![ReplayEvent::new(
        http::Request::builder()
            .uri(list_uri("test-bucket", "uploads/"))
            .method("GET")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder()
            .status(200)
            .body(SdkBody::from(list_response_xml(empty, false)))
            .unwrap(),
    )];

    let client = mock_client(events);
    let result = cleanup_orphaned_uploads(&client, "test-bucket", "uploads/", cutoff, 1).await;
    assert!(result.is_ok(), "empty list should succeed");
}

/// Upload newer than cutoff → skipped, not aborted.
#[tokio::test]
async fn test_new_upload_is_skipped() {
    let recent = Utc::now() - Duration::hours(1);
    let recent_str = DateTime::from_secs(recent.timestamp())
        .fmt(Format::DateTime)
        .unwrap();
    let cutoff = recent - Duration::hours(1); // 2h ago → upload is newer (1h ago)

    let events = vec![ReplayEvent::new(
        http::Request::builder()
            .uri(list_uri("test-bucket", "uploads/"))
            .method("GET")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder()
            .status(200)
            .body(SdkBody::from(list_response_xml(
                &[("uploads/abc", "upload-1", &recent_str)],
                false,
            )))
            .unwrap(),
    )];

    let client = mock_client(events);
    let result = cleanup_orphaned_uploads(&client, "test-bucket", "uploads/", cutoff, 1).await;
    assert!(result.is_ok(), "newer upload should be skipped silently");
}

/// Upload older than cutoff → aborted.
#[tokio::test]
async fn test_old_upload_is_aborted() {
    let old = Utc::now() - Duration::hours(24);
    let old_str = DateTime::from_secs(old.timestamp())
        .fmt(Format::DateTime)
        .unwrap();
    let cutoff = Utc::now() - Duration::hours(6);

    let events = vec![
        // First: list returns the old upload
        ReplayEvent::new(
            http::Request::builder()
                .uri(list_uri("test-bucket", "uploads/"))
                .method("GET")
                .body(SdkBody::empty())
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(list_response_xml(
                    &[("uploads/old-file", "upload-old", &old_str)],
                    false,
                )))
                .unwrap(),
        ),
        // Second: abort is called for the old upload
        abort_event("test-bucket", "uploads/old-file", "upload-old"),
    ];

    let client = mock_client(events);
    let result = cleanup_orphaned_uploads(&client, "test-bucket", "uploads/", cutoff, 1).await;
    assert!(result.is_ok(), "old upload should be aborted successfully");
}

/// Mix of old and new uploads → only old ones aborted.
#[tokio::test]
async fn test_mixed_old_and_new_uploads() {
    let old = Utc::now() - Duration::hours(48);
    let recent = Utc::now() - Duration::hours(1);
    let old_str = DateTime::from_secs(old.timestamp())
        .fmt(Format::DateTime)
        .unwrap();
    let recent_str = DateTime::from_secs(recent.timestamp())
        .fmt(Format::DateTime)
        .unwrap();
    let cutoff = Utc::now() - Duration::hours(6);

    let events = vec![
        ReplayEvent::new(
            http::Request::builder()
                .uri(list_uri("test-bucket", "uploads/"))
                .method("GET")
                .body(SdkBody::empty())
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(list_response_xml(
                    &[
                        ("uploads/old", "upload-old", &old_str),
                        ("uploads/recent", "upload-recent", &recent_str),
                    ],
                    false,
                )))
                .unwrap(),
        ),
        abort_event("test-bucket", "uploads/old", "upload-old"),
    ];

    let client = mock_client(events);
    let result = cleanup_orphaned_uploads(&client, "test-bucket", "uploads/", cutoff, 1).await;
    assert!(result.is_ok(), "only old uploads should be aborted");
}

/// Pagination: truncated response → follow pagination markers.
#[tokio::test]
async fn test_pagination_follows_markers() {
    let old = Utc::now() - Duration::hours(24);
    let old_str = DateTime::from_secs(old.timestamp())
        .fmt(Format::DateTime)
        .unwrap();
    let cutoff = Utc::now() - Duration::hours(6);

    let events = vec![
        // Page 1: truncated, one upload
        ReplayEvent::new(
            http::Request::builder()
                .uri(list_uri("test-bucket", "uploads/"))
                .method("GET")
                .body(SdkBody::empty())
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(list_response_xml(
                    &[("uploads/page1", "up1", &old_str)],
                    true,
                )))
                .unwrap(),
        ),
        abort_event("test-bucket", "uploads/page1", "up1"),
        // Page 2: not truncated, one upload
        ReplayEvent::new(
            http::Request::builder()
                .uri(list_uri("test-bucket", "uploads/"))
                .method("GET")
                .body(SdkBody::empty())
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(list_response_xml(
                    &[("uploads/page2", "up2", &old_str)],
                    false,
                )))
                .unwrap(),
        ),
        abort_event("test-bucket", "uploads/page2", "up2"),
    ];

    let client = mock_client(events);
    let result = cleanup_orphaned_uploads(&client, "test-bucket", "uploads/", cutoff, 1).await;
    assert!(result.is_ok(), "pagination should process all pages");
}

/// Concurrency: multiple old uploads are all aborted.
#[tokio::test]
async fn test_concurrent_aborts() {
    let old = Utc::now() - Duration::hours(48);
    let old_str = DateTime::from_secs(old.timestamp())
        .fmt(Format::DateTime)
        .unwrap();
    let cutoff = Utc::now() - Duration::hours(6);

    // Build upload data
    let keys: Vec<String> = (0..3).map(|i| format!("uploads/file-{}", i)).collect();
    let ids: Vec<String> = (0..3).map(|i| format!("upload-{}", i)).collect();
    let upload_refs: Vec<(&str, &str, &str)> = keys
        .iter()
        .zip(ids.iter())
        .map(|(k, u)| (k.as_str(), u.as_str(), old_str.as_str()))
        .collect();

    let mut events = vec![
        // List returns 3 old uploads
        ReplayEvent::new(
            http::Request::builder()
                .uri(list_uri("test-bucket", "uploads/"))
                .method("GET")
                .body(SdkBody::empty())
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(list_response_xml(&upload_refs, false)))
                .unwrap(),
        ),
    ];

    // All 3 are aborted
    for (key, upload_id) in keys.iter().zip(ids.iter()) {
        events.push(abort_event("test-bucket", key, upload_id));
    }

    let client = mock_client(events);
    let result =
        cleanup_orphaned_uploads(&client, "test-bucket", "uploads/", cutoff, MAX_CONCURRENT).await;
    assert!(
        result.is_ok(),
        "all old uploads should be aborted concurrently"
    );
}
