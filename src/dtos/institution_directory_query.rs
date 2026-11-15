use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchInstitutionsQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
}
