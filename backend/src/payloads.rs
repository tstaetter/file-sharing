use serde::{Deserialize, Serialize};

use crate::user::User;

//---- AbortUploads ----//

#[derive(Debug, Deserialize)]
pub struct AbortRequest {
    pub key: String,
    pub upload_id: String,
}

//---- Auth ----//

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub email: String,
    pub name: String,
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        UserResponse {
            email: user.email.clone(),
            name: user.name.clone(),
        }
    }
}

//---- CheckFile ----//

#[derive(Debug, Deserialize)]
pub struct CheckFileRequest {
    pub key: String,
}

//---- CompleteUpload ----//

#[derive(Debug, Deserialize)]
pub struct CompleteRequest {
    pub key: String,
    pub upload_id: String,
    pub parts: Vec<PartETag>,
}

#[derive(Debug, Deserialize)]
pub struct PartETag {
    pub part_number: i32,
    pub etag: String,
}

//---- CreateUpload ----//

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateRequest {
    pub file_id: String,
    pub content_type: Option<String>,
    pub chunk_size: Option<u64>,
}

#[derive(Serialize)]
pub struct CreateResponse {
    pub upload_id: String,
    pub key: String,
}

//---- SignParts ----//

#[derive(Debug, Deserialize)]
pub struct SignPartsRequest {
    pub key: String,
    pub upload_id: String,
    pub part_numbers: Vec<i32>,
}

#[derive(Serialize)]
pub struct SignedPart {
    pub part_number: i32,
    pub url: String,
}
