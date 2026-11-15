use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::research::ResearchProjectVisibility,
    errors::app_error::AppError,
    models::research_project::ResearchProject,
    repositories::research_project_repository::{self, NewResearchProject},
    validations::research_validation,
};

pub struct CreateResearchProjectDetails {
    pub title: String,
    pub abstract_text: Option<String>,
    pub description: String,
    pub objectives: Vec<String>,
    pub research_questions: Vec<String>,
    pub subject_ids: Vec<Uuid>,
    pub visibility: ResearchProjectVisibility,
}

pub async fn create_research_project(
    pool: &PgPool,
    owner_account_id: Uuid,
    details: CreateResearchProjectDetails,
) -> Result<ResearchProject, AppError> {
    let title = research_validation::validate_research_project_title(&details.title)?;
    let abstract_text =
        research_validation::validate_research_project_abstract(details.abstract_text.as_deref())?;
    let description =
        research_validation::validate_research_project_description(&details.description)?;
    let objectives =
        research_validation::validate_research_project_objectives(&details.objectives)?;
    let research_questions =
        research_validation::validate_research_questions(&details.research_questions)?;
    research_validation::validate_research_subject_ids(&details.subject_ids)?;

    research_project_repository::create_research_project(
        pool,
        owner_account_id,
        Uuid::new_v4(),
        NewResearchProject {
            title: &title,
            abstract_text: abstract_text.as_deref(),
            description: &description,
            objectives: &objectives,
            research_questions: &research_questions,
            subject_ids: &details.subject_ids,
            visibility: details.visibility.as_database_value(),
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::CreateResearchProjectDetails;
    use crate::enums::research::ResearchProjectVisibility;

    #[test]
    fn research_projects_default_to_private_visibility() {
        let details = CreateResearchProjectDetails {
            title: "Research title".to_owned(),
            abstract_text: None,
            description: String::new(),
            objectives: Vec::new(),
            research_questions: Vec::new(),
            subject_ids: Vec::new(),
            visibility: ResearchProjectVisibility::default(),
        };
        assert_eq!(details.visibility, ResearchProjectVisibility::Private);
    }
}
