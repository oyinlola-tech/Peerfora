use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::ResearchProjectError,
    errors::app_error::AppError,
    models::research_project::ResearchProject,
    repositories::research_project_repository::{self, ProjectUpdate},
    services::research::queries::find_research_project_query,
    validations::research_validation,
};

pub struct UpdateResearchProjectDetails {
    pub title: Option<String>,
    pub abstract_text: Option<Option<String>>,
    pub description: Option<String>,
    pub objectives: Option<Vec<String>>,
    pub research_questions: Option<Vec<String>>,
    pub subject_ids: Option<Vec<Uuid>>,
}

pub async fn update_research_project(
    pool: &PgPool,
    project_id: Uuid,
    owner_account_id: Uuid,
    details: UpdateResearchProjectDetails,
) -> Result<ResearchProject, AppError> {
    let existing_project = find_research_project_query::find_research_project_for_account(
        pool,
        project_id,
        Some(owner_account_id),
    )
    .await?
    .ok_or(ResearchProjectError::ProjectNotFound)?;
    if existing_project.current_member_role.as_deref() != Some("owner") {
        return Err(ResearchProjectError::PermissionDenied.into());
    }

    let title = research_validation::validate_research_project_title(
        details.title.as_deref().unwrap_or(&existing_project.title),
    )?;
    let abstract_text = match details.abstract_text {
        Some(abstract_text) => {
            research_validation::validate_research_project_abstract(abstract_text.as_deref())?
        }
        None => existing_project.abstract_text,
    };
    let description = research_validation::validate_research_project_description(
        details
            .description
            .as_deref()
            .unwrap_or(&existing_project.description),
    )?;
    let objectives = research_validation::validate_research_project_objectives(
        details
            .objectives
            .as_deref()
            .unwrap_or(&existing_project.objectives),
    )?;
    let research_questions = research_validation::validate_research_questions(
        details
            .research_questions
            .as_deref()
            .unwrap_or(&existing_project.research_questions),
    )?;
    let subject_ids = details.subject_ids.unwrap_or(existing_project.subject_ids);
    research_validation::validate_research_subject_ids(&subject_ids)?;

    research_project_repository::update_research_project(
        pool,
        project_id,
        owner_account_id,
        ProjectUpdate {
            title: &title,
            abstract_text: abstract_text.as_deref(),
            description: &description,
            objectives: &objectives,
            research_questions: &research_questions,
            subject_ids: &subject_ids,
        },
    )
    .await
}
