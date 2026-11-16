use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    dtos::academic_directory_requests::SearchAcademicSubjectsRequest,
    errors::app_error::AppError,
    models::{academic_directory::AcademicSubject, app_state::AppState},
    services::academic_directory::queries::search_academic_subjects_query,
};

pub async fn search_subjects(
    State(state): State<AppState>,
    Query(query): Query<SearchAcademicSubjectsRequest>,
) -> Result<Json<Vec<AcademicSubject>>, AppError> {
    let subjects = search_academic_subjects_query::search_academic_subjects(
        &state.database,
        query.q.as_deref(),
        query.limit,
    )
    .await?;
    Ok(Json(subjects))
}
