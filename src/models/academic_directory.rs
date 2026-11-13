use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct AcademicSubject {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub discipline_name: Option<String>,
}

#[derive(FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub country_code: String,
    pub alpha_3: String,
    pub display_name: String,
    pub official_name: Option<String>,
}

#[derive(FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicLevel {
    pub id: String,
    pub display_name: String,
    pub display_order: i16,
    pub aliases: Vec<String>,
}
