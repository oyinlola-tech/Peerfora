use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListCommunitiesQuery {
    pub limit: Option<i64>,
}
