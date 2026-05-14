use crate::handlers::{
    abort_upload, check_file, complete_upload, create_upload, delete_user, download, health, login,
    register, sign_parts,
};
use crate::middleware::require_auth;
use crate::AppState;
use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;

pub fn app(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state.clone());
    let protected_routes = Router::new()
        .route("/delete", delete(delete_user))
        .layer(middleware::from_fn(require_auth));
    let routes = Router::new()
        .route("/create-upload", post(create_upload))
        .route("/check-file", put(check_file))
        .route("/sign-parts", post(sign_parts))
        .route("/complete-upload", post(complete_upload))
        .route("/abort-upload", post(abort_upload))
        .route("/f/{id}", get(download))
        .nest("/auth", auth_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .with_state(state);

    Router::new()
        .route("/health", get(health))
        .nest("/v1", routes)
}
