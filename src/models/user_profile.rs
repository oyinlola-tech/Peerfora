use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

pub struct UserProfileChanges {
    pub display_name: Option<String>,
    pub biography: Option<String>,
    pub avatar_reference: Option<String>,
    pub banner_reference: Option<String>,
    pub academic_level_id: Option<String>,
    pub institution_id: Option<Uuid>,
    pub discipline_id: Option<Uuid>,
    pub profile_visibility: Option<String>,
    pub show_academic_affiliation: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserProfile {
    pub username: String,
    pub display_name: Option<String>,
    pub biography: String,
    pub avatar_reference: Option<String>,
    pub banner_reference: Option<String>,
    pub academic_level_id: Option<String>,
    pub institution_name: Option<String>,
    pub discipline_name: Option<String>,
    pub show_academic_affiliation: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnUserProfile {
    pub account_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub biography: String,
    pub avatar_reference: Option<String>,
    pub banner_reference: Option<String>,
    pub academic_level_id: Option<String>,
    pub institution_id: Option<Uuid>,
    pub discipline_id: Option<Uuid>,
    pub profile_visibility: String,
    pub show_academic_affiliation: bool,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPreferences {
    pub language_tag: String,
    pub country_code: String,
    pub notification_preferences: serde_json::Value,
    pub content_preferences: serde_json::Value,
    pub personalization_enabled: bool,
}
