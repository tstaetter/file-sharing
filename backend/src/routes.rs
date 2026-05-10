use crate::{abort_upload, complete_upload, create_upload, download, health, sign_parts, AppState};
use axum::routing::get;
use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;

pub fn app(state: AppState) -> Router {
    let routes = Router::new()
        .route("/create-upload", post(create_upload))
        .route("/sign-parts", post(sign_parts))
        .route("/complete-upload", post(complete_upload))
        .route("/abort-upload", post(abort_upload))
        .route("/f/{id}", get(download))
        .layer(CorsLayer::permissive())
        .with_state(state);

    Router::new()
        .route("/health", get(health))
        .nest("/v1", routes)
}
