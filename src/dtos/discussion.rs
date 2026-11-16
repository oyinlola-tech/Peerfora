use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    enums::discussion::CommunityPostKind,
    models::discussion::{CommunityPost, CommunityPostComment},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommunityPostRequest {
    pub kind: CommunityPostKind,
    pub title: String,
    pub body: String,
}

#[derive(Deserialize)]
pub struct ListCommunityPostsQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostCommentRequest {
    pub body: String,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct ListPostCommentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePostResearchFindingReferenceRequest {
    pub finding_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityPostResponse {
    pub id: Uuid,
    pub community_slug: String,
    pub author_username: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CommunityPost> for CommunityPostResponse {
    fn from(post: CommunityPost) -> Self {
        Self {
            id: post.id,
            community_slug: post.community_slug,
            author_username: post.author_username,
            kind: post.kind,
            title: post.title,
            body: post.body,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostCommentResponse {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub author_username: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CommunityPostComment> for PostCommentResponse {
    fn from(comment: CommunityPostComment) -> Self {
        Self {
            id: comment.id,
            post_id: comment.post_id,
            parent_comment_id: comment.parent_comment_id,
            author_username: comment.author_username,
            body: comment.body,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        }
    }
}
