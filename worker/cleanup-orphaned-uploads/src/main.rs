use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;
use chrono::Duration;
use cleanup_orphaned_uploads::{cleanup_orphaned_uploads, CleanupResult, DEFAULT_ORPHAN_AGE_HOURS};
use std::env;
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};

const MAX_CONCURRENT: usize = cleanup_orphaned_uploads::MAX_CONCURRENT;

#[tokio::main]
async fn main() -> CleanupResult<()> {
    // eprintln goes to stderr and is visible even before tracing is initialised.
    // This confirms the binary actually started executing.
    eprintln!("cleanup-orphaned-uploads: process started, initialising...");

    dotenvy::dotenv().ok();

    // Default to info-level logging if RUST_LOG is not set.
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

    let cutoff = chrono::Utc::now() - Duration::hours(DEFAULT_ORPHAN_AGE_HOURS);

    cleanup_orphaned_uploads(&client, &bucket, "uploads/", cutoff, MAX_CONCURRENT).await
}
