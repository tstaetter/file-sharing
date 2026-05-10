use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{Duration, Utc};
use cleanup_orphaned_uploads::{CleanupError, CleanupResult};
use futures::{StreamExt, stream::FuturesUnordered};
use std::env;
use tracing::info;
use tracing_subscriber::{EnvFilter, prelude::*};

const MAX_CONCURRENT: usize = 10;

#[tokio::main]
async fn main() -> CleanupResult<()> {
    // eprintln goes to stderr and is visible even before tracing is initialised.
    // This confirms the binary actually started executing.
    eprintln!("cleanup-orphaned-uploads: process started, initialising...");

    dotenvy::dotenv().ok();

    // Default to info-level logging if RUST_LOG is not set.
    // Without this, EnvFilter::from_default_env() produces no output at all
    // when RUST_LOG is absent, making startup failures invisible on Koyeb.
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // Init tracing, write logs to STDOUT.
    // Disable ANSI colors in Docker containers (no TTY) to avoid garbled output.
    let is_tty = atty::is(atty::Stream::Stdout);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(is_tty)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_writer(std::io::stdout),
        )
        .init();

    info!("Cleanup worker starting...");

    let account_id = env::var("R2_ACCOUNT_ID").expect("R2_ACCOUNT_ID must be set");
    let access_key = env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID must be set");
    let secret_key = env::var("R2_SECRET_ACCESS_KEY").expect("R2_SECRET_ACCESS_KEY must be set");
    let bucket = env::var("R2_BUCKET").expect("R2_BUCKET must be set");

    info!(
        bucket = %bucket,
        account_id = %account_id,
        "Connecting to Cloudflare R2"
    );

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

    info!("S3 client initialized");

    // cutoff: alles älter als X Stunden löschen
    let cutoff = Utc::now() - Duration::hours(6);

    info!("Cleaning uploads older than {}", cutoff);

    let mut tasks = FuturesUnordered::new();
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
            let key = upload.key().unwrap_or_default().to_string();
            let upload_id = upload.upload_id().unwrap_or_default().to_string();

            let initiated = upload.initiated().unwrap();
            let initiated_time = initiated.to_chrono_utc()?;

            if initiated_time < cutoff {
                let client_clone = client.clone();
                let bucket_clone = bucket.clone();

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

                // 🧠 Concurrency Limit
                if tasks.len() >= MAX_CONCURRENT {
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

    // remaining tasks abwarten
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
