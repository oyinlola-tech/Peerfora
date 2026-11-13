use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct Community {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub institution_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub discipline_id: Option<Uuid>,
    pub owner_account_id: Uuid,
    pub visibility: String,
    pub membership_policy: String,
    pub lifecycle_status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow, Serialize)]
pub struct CommunityMembership {
    pub community_id: Uuid,
    pub account_id: Uuid,
    pub membership_status: String,
    pub community_role: String,
    pub joined_at: Option<chrono::DateTime<chrono::Utc>>,
}
