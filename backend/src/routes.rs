use crate::{AppState, abort_upload, complete_upload, create_upload, download, sign_parts};
use axum::routing::get;
use axum::{Router, routing::post};
use tower_http::cors::CorsLayer;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/create-upload", post(create_upload))
        .route("/sign-parts", post(sign_parts))
        .route("/complete-upload", post(complete_upload))
        .route("/abort-upload", post(abort_upload))
        .route("/f/{id}", get(download))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
