use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::community::{Community, CommunityMembership};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityResponse {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub institution_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub discipline_id: Option<Uuid>,
    pub visibility: String,
    pub membership_policy: String,
    pub lifecycle_status: String,
    pub created_at: DateTime<Utc>,
}

impl From<Community> for CommunityResponse {
    fn from(community: Community) -> Self {
        Self {
            id: community.id,
            slug: community.slug,
            display_name: community.display_name,
            description: community.description,
            category: community.category,
            institution_id: community.institution_id,
            course_id: community.course_id,
            discipline_id: community.discipline_id,
            visibility: community.visibility,
            membership_policy: community.membership_policy,
            lifecycle_status: community.lifecycle_status,
            created_at: community.created_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityMembershipResponse {
    pub community_id: Uuid,
    pub membership_status: String,
    pub community_role: String,
    pub joined_at: Option<DateTime<Utc>>,
}

impl From<CommunityMembership> for CommunityMembershipResponse {
    fn from(membership: CommunityMembership) -> Self {
        Self {
            community_id: membership.community_id,
            membership_status: membership.membership_status,
            community_role: membership.community_role,
            joined_at: membership.joined_at,
        }
    }
}
