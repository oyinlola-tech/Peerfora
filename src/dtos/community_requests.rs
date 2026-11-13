use serde::Deserialize;
use uuid::Uuid;

use crate::enums::community::{CommunityCategory, CommunityMembershipPolicy, CommunityVisibility};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommunityRequest {
    pub slug: Option<String>,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: CommunityCategory,
    #[serde(default)]
    pub visibility: CommunityVisibility,
    #[serde(default)]
    pub membership_policy: CommunityMembershipPolicy,
    #[serde(default = "default_language_tag")]
    pub primary_language_tag: String,
    pub institution_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub discipline_id: Option<Uuid>,
    #[serde(default)]
    pub subject_ids: Vec<Uuid>,
}

fn default_language_tag() -> String {
    "en".to_owned()
}
