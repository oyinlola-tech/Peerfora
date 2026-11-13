use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOwnProfileRequest {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountPreferencesRequest {
    pub language_tag: Option<String>,
    pub country_code: Option<String>,
    pub notification_preferences: Option<serde_json::Value>,
    pub content_preferences: Option<serde_json::Value>,
    pub personalization_enabled: Option<bool>,
}
