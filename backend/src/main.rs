use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_sdk_s3::types::{CorsConfiguration, CorsRule};
use backend::*;
use std::env;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, prelude::*};

#[tokio::main]
async fn main() {
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

    let s3 = Client::from_conf(config);

    // Apply CORS configuration to the R2 bucket so browsers can upload/download directly.
    let cors_rule = CorsRule::builder()
        .allowed_origins("*")
        .allowed_methods("GET")
        .allowed_methods("PUT")
        .allowed_methods("HEAD")
        .allowed_headers("*")
        .expose_headers("ETag")
        .expose_headers("x-amz-request-id")
        .expose_headers("x-amz-id-2")
        .max_age_seconds(3600)
        .build()
        .expect("failed to build CORS rule");

    let cors_config = CorsConfiguration::builder()
        .cors_rules(cors_rule)
        .build()
        .expect("failed to build CORS configuration");

    match s3
        .put_bucket_cors()
        .bucket(&bucket)
        .cors_configuration(cors_config)
        .send()
        .await
    {
        Ok(_) => info!("R2 bucket CORS policy applied"),
        Err(err) => {
            let svc = err.into_service_error();
            error!(
                "warning: failed to set R2 bucket CORS policy: {} (code={:?})",
                svc,
                svc.code()
            );
        }
    }

    let state = AppState { s3, bucket };

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app(state))
        .await
        .expect("server error");
}
