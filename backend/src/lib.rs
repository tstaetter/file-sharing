mod db;
mod handlers;
mod middleware;
mod routes;

pub use db::*;
pub use handlers::*;
pub use middleware::*;
pub use routes::app;

use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct AppState {
    pub s3: Client,
    pub bucket: String,
    pub database: Option<mongodb::Database>,
}
