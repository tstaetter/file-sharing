use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::Model;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SavedUrlId {
    Uuid(Uuid),
    ObjectId(ObjectId),
}

impl fmt::Display for SavedUrlId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uuid(u) => write!(f, "{}", u),
            Self::ObjectId(o) => write!(f, "{}", o),
        }
    }
}

impl From<Uuid> for SavedUrlId {
    fn from(u: Uuid) -> Self {
        Self::Uuid(u)
    }
}

impl From<ObjectId> for SavedUrlId {
    fn from(o: ObjectId) -> Self {
        Self::ObjectId(o)
    }
}

impl From<SavedUrlId> for mongodb::bson::Bson {
    fn from(id: SavedUrlId) -> Self {
        match id {
            SavedUrlId::Uuid(u) => mongodb::bson::Bson::from(u),
            SavedUrlId::ObjectId(o) => mongodb::bson::Bson::from(o),
        }
    }
}

/// Represents a capability URL saved by an authenticated user.
///
/// Each saved URL belongs to exactly one user (identified by user_email)
/// and stores the full capability URL along with an optional descriptive title
/// and the timestamp when it was saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedUrl {
    /// Unique identifier for this saved URL record.
    #[serde(rename = "_id", default = "default_id")]
    pub id: SavedUrlId,

    /// Email address of the user who saved this URL.
    /// Used to scope listing queries - a user can only see their own URLs.
    pub user_email: String,

    /// The full capability URL (e.g. https://filez.zone/f/uuid#key).
    pub url: String,

    /// Optional human-readable title or description for this link
    /// (e.g. "Vacation photos", "Contract draft v3").
    #[serde(default)]
    pub title: Option<String>,

    /// ISO-8601 timestamp of when this URL was saved.
    /// Stored as a chrono::DateTime<Utc>; serialized to/from BSON date type
    /// via the bson crate's chrono support.
    #[serde(default = "Utc::now", with = "bson_datetime")]
    pub created_at: DateTime<Utc>,
}

mod bson_datetime {
    use chrono::{DateTime, Utc};
    use mongodb::bson::Bson;
    use mongodb::bson::DateTime as BsonDateTime;
    use serde::de::Error;
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        BsonDateTime::from_chrono(*date).serialize(serializer)
    }

    /// Deserializes a BSON value into a `DateTime<Utc>`.
    ///
    /// Handles three cases gracefully:
    /// - **BSON DateTime** — the expected storage format.
    /// - **BSON String**   — ISO‑8601 / RFC‑3339 string (e.g. from manual inserts
    ///   or data migration).
    /// - **BSON Null**     — treated as missing; returns `Utc::now()` as a
    ///   safe default.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bson = Bson::deserialize(deserializer)?;
        match bson {
            Bson::DateTime(dt) => Ok(dt.to_chrono()),
            Bson::String(s) => {
                // Try RFC 3339 / ISO 8601 with flexible sub-second precision.
                s.parse::<DateTime<Utc>>().map_err(|e| {
                    D::Error::custom(format!("invalid datetime string '{}': {}", s, e))
                })
            }
            Bson::Null => Ok(Utc::now()),
            other => Err(D::Error::custom(format!(
                "expected DateTime, got {:?}",
                other.element_type()
            ))),
        }
    }
}

fn default_id() -> SavedUrlId {
    SavedUrlId::Uuid(Uuid::new())
}

impl SavedUrl {
    /// Creates a new SavedUrl with a freshly generated UUID, the current
    /// timestamp, and the given field values.
    pub fn new(user_email: String, url: String, title: Option<String>) -> Self {
        Self {
            id: default_id(),
            user_email,
            url,
            title,
            created_at: Utc::now(),
        }
    }
}

impl Model for SavedUrl {}
