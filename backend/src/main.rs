use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_sdk_s3::types::{CorsConfiguration, CorsRule};
use aws_sdk_s3::Client;
use backend::*;
use std::env;
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    // eprintln goes to stderr and is visible even before tracing is initialised.
    // This confirms the binary actually started executing.
    eprintln!("backend: process started, initialising...");

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

    info!("Backend starting up...");

    // Validate required environment variables early
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

    let s3 = Client::from_conf(config);

    info!("S3 client initialized");

    // Apply CORS configuration to the R2 bucket in a background task.
    // We do NOT block startup on this — if it fails, the bucket may already
    // have CORS configured, and the server can still function without it.
    // On platforms like Koyeb, a slow R2 call can cause the container to be
    // killed for exceeding the startup timeout, so we fire-and-forget.
    let cors_s3 = s3.clone();
    let cors_bucket = bucket.clone();

    let cors_task = tokio::spawn(async move {
        let cors_rule = CorsRule::builder()
            .allowed_origins("*")
            .allowed_methods("GET")
            .allowed_methods("PUT")
            .allowed_methods("HEAD")
            .allowed_headers("*")
            .expose_headers("ETag")
            .expose_headers("x-amz-request-id")
            .expose_headers("x-amz-id-2")
            .expose_headers("x-chunk-size")
            .expose_headers("x-content-type")
            .max_age_seconds(3600)
            .build()
            .expect("failed to build CORS rule");

        let cors_config = CorsConfiguration::builder()
            .cors_rules(cors_rule)
            .build()
            .expect("failed to build CORS configuration");

        match cors_s3
            .put_bucket_cors()
            .bucket(&cors_bucket)
            .cors_configuration(cors_config)
            .send()
            .await
        {
            Ok(_) => info!("R2 bucket CORS policy applied"),
            Err(err) => {
                let svc = err.into_service_error();
                warn!(
                    "Failed to set R2 bucket CORS policy: {} (code={:?}). \
                     The bucket may already have CORS configured — server will continue.",
                    svc,
                    svc.code()
                );
            }
        }
    });

    let state = AppState { s3, bucket, database: None };

    // Koyeb sets PORT at runtime; fall back to 8000 for local dev
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    info!("Binding to 0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("failed to bind TCP listener");

    info!("Server listening on 0.0.0.0:{}, health at /health", port);

    // Graceful shutdown: listen for SIGTERM (Koyeb) or SIGINT (Ctrl+C locally)
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");

    info!("Server shutting down gracefully");

    // Wait for the background CORS task to finish before exiting
    if let Err(err) = cors_task.await {
        warn!("CORS background task panicked: {:?}", err);
    }
}

/// Returns a future that completes when a shutdown signal is received.
/// Handles SIGTERM (sent by Koyeb and other container orchestrators)
/// and SIGINT (Ctrl+C in local development).
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            info!("Received SIGINT, shutting down...");
        },
        () = terminate => {
            info!("Received SIGTERM, shutting down...");
        },
    }
}
