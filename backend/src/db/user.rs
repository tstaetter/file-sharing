use serde::{Deserialize, Serialize};

use crate::Model;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub email: String,
    pub password_hash: String,
    pub name: String,
}

impl Model for User {}
