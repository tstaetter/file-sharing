use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{Duration, Utc};
use cleanup_orphaned_uploads::{CleanupError, CleanupResult};
use std::env;
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> CleanupResult<()> {
    dotenvy::dotenv().ok();

    // Init tracing, write logs to STDOUT
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_writer(std::io::stdout),
        )
        .init();

    let account_id = env::var("R2_ACCOUNT_ID").expect("R2_ACCOUNT_ID must be set");
    let access_key = env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID must be set");
    let secret_key = env::var("R2_SECRET_ACCESS_KEY").expect("R2_SECRET_ACCESS_KEY must be set");
    let bucket = env::var("R2_BUCKET").expect("R2_BUCKET must be set");
    let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", account_id);
    let credentials = Credentials::new(access_key, secret_key, None, None, "cloudflare-r2");
    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .endpoint_url(endpoint_url)
        .region(Region::new("auto"))
        .force_path_style(true)
        .credentials_provider(credentials)
        .build();
    let client = Client::from_conf(config);
    // cutoff: alles älter als X Stunden löschen
    let cutoff = Utc::now() - Duration::hours(6);

    info!("Cleaning uploads older than {}", cutoff);

    let mut key_marker = None;
    let mut upload_id_marker = None;

    loop {
        let resp = client
            .list_multipart_uploads()
            .prefix("uploads/")
            .bucket(&bucket)
            .set_key_marker(key_marker.clone())
            .set_upload_id_marker(upload_id_marker.clone())
            .send()
            .await
            .map_err(|e| CleanupError::Sdk(e.to_string()))?;

        for upload in resp.uploads() {
            let key = upload.key().unwrap_or_default();
            let upload_id = upload.upload_id().unwrap_or_default();

            let initiated = upload.initiated().unwrap();
            let initiated_time = initiated.to_chrono_utc()?;

            if initiated_time < cutoff {
                info!("Aborting upload: {} ({})", key, upload_id);

                client
                    .abort_multipart_upload()
                    .bucket(&bucket)
                    .key(key)
                    .upload_id(upload_id)
                    .send()
                    .await
                    .map_err(|e| CleanupError::Sdk(e.to_string()))?;
            }
        }

        // pagination check
        if let Some(truncated) = resp.is_truncated() {
            if truncated {
                key_marker = resp.next_key_marker().map(|s| s.to_string());
                upload_id_marker = resp.next_upload_id_marker().map(|s| s.to_string());
            } else {
                break;
            }
        } else {
            break;
        }
    }

    info!("Cleanup finished");

    Ok(())
}
