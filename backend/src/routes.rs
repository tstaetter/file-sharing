use crate::handlers::{
    abort_upload, complete_upload, create_upload, delete_user, download, health, list_urls, login,
    register, save_url, sign_parts,
};
use crate::AppState;
use axum::routing::get;
use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;

pub fn app(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/delete", post(delete_user))
        .with_state(state.clone());
    let protected_routes = Router::new().route("/urls", post(save_url).get(list_urls));
    let routes = Router::new()
        .route("/create-upload", post(create_upload))
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
