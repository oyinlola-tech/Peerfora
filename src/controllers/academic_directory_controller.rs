use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    dtos::academic_directory_requests::{
        SearchAcademicDirectoryRequest, SearchAcademicSubjectsRequest,
    },
    errors::app_error::AppError,
    models::{academic_directory::AcademicSubject, app_state::AppState},
    services::academic_directory::queries::search_academic_subjects_query,
};

pub async fn search_academic_subjects(
    State(state): State<AppState>,
    Query(request): Query<SearchAcademicSubjectsRequest>,
) -> Result<Json<Vec<AcademicSubject>>, AppError> {
    let subjects = search_academic_subjects_query::search_academic_subjects(
        &state.database,
        request.q.as_deref(),
        request.limit,
    )
    .await?;
    Ok(Json(subjects))
}

pub async fn search_countries(
    State(state): State<AppState>,
    Query(request): Query<SearchAcademicDirectoryRequest>,
) -> Result<Json<Vec<crate::models::academic_directory::Country>>, AppError> {
    let countries =
        crate::services::academic_directory::queries::search_countries_query::search_countries(
            &state.database,
            request.q.as_deref(),
            request.limit,
        )
        .await?;
    Ok(Json(countries))
}

pub async fn list_academic_levels(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::academic_directory::AcademicLevel>>, AppError> {
    let levels = crate::services::academic_directory::queries::list_academic_levels_query::list_academic_levels(
        &state.database,
    )
    .await?;
    Ok(Json(levels))
}
