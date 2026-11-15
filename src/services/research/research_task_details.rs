use crate::{enums::research::ResearchTaskError, validations::research_validation};

pub struct ResearchTaskDetails {
    pub title: String,
    pub objective: String,
    pub scope: String,
    pub allowed_data_sources: Vec<String>,
    pub permitted_tools: Vec<String>,
    pub expected_deliverables: Vec<String>,
    pub max_runtime_seconds: i32,
    pub max_result_bytes: i32,
}

pub fn validate_research_task_details(
    details: ResearchTaskDetails,
) -> Result<ResearchTaskDetails, ResearchTaskError> {
    Ok(ResearchTaskDetails {
        title: research_validation::validate_research_task_title(&details.title)?,
        objective: research_validation::validate_research_task_objective(&details.objective)?,
        scope: research_validation::validate_research_task_scope(&details.scope)?,
        allowed_data_sources: research_validation::validate_research_task_data_sources(
            &details.allowed_data_sources,
        )?,
        permitted_tools: research_validation::validate_research_task_permitted_tools(
            &details.permitted_tools,
        )?,
        expected_deliverables: research_validation::validate_research_task_deliverables(
            &details.expected_deliverables,
        )?,
        max_runtime_seconds: {
            research_validation::validate_research_task_runtime_limit(details.max_runtime_seconds)?;
            details.max_runtime_seconds
        },
        max_result_bytes: {
            research_validation::validate_research_task_result_limit(details.max_result_bytes)?;
            details.max_result_bytes
        },
    })
}
