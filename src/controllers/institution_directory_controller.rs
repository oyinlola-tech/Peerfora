use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    dtos::institution_directory_query::SearchInstitutionsQuery,
    errors::app_error::AppError,
    models::{academic_directory::InstitutionDirectoryEntry, app_state::AppState},
    services::institutions::queries::search_institutions_query,
};

pub async fn search_institutions(
    State(state): State<AppState>,
    Query(request): Query<SearchInstitutionsQuery>,
) -> Result<Json<Vec<InstitutionDirectoryEntry>>, AppError> {
    let institutions = search_institutions_query::search_institutions(
        &state.database,
        request.q.as_deref(),
        request.limit,
    )
    .await?;
    Ok(Json(institutions))
}
