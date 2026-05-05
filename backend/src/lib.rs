mod handlers;
mod routes;

pub use handlers::*;
pub use routes::app;

use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct AppState {
    pub s3: Client,
    pub bucket: String,
}
