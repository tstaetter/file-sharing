use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Model;

/// Represents a capability URL saved by an authenticated user.
///
/// Each saved URL belongs to exactly one user (identified by `user_email`)
/// and stores the full capability URL along with an optional descriptive title
/// and the timestamp when it was saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedUrl {
    /// Unique identifier for this saved URL record.
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Email address of the user who saved this URL.
    /// Used to scope listing queries — a user can only see their own URLs.
    pub user_email: String,

    /// The full capability URL (e.g. `https://filez.zone/f/uuid#key`).
    pub url: String,

    /// Optional human-readable title or description for this link
    /// (e.g. "Vacation photos", "Contract draft v3").
    #[serde(default)]
    pub title: Option<String>,

    /// ISO-8601 timestamp of when this URL was saved.
    /// Stored as a `chrono::DateTime<Utc>`; serialized to/from BSON date type
    /// via the `bson` crate's chrono support.
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

impl SavedUrl {
    /// Creates a new `SavedUrl` with a freshly generated UUID, the current
    /// timestamp, and the given field values.
    pub fn new(user_email: String, url: String, title: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_email,
            url,
            title,
            created_at: Utc::now(),
        }
    }
}

impl Model for SavedUrl {}
