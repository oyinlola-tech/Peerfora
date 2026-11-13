use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchAcademicSubjectsRequest {
    pub q: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchAcademicDirectoryRequest {
    pub q: Option<String>,
    pub limit: Option<i64>,
}
