use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    constants::research,
    enums::research::{ResearchProjectError, ResearchTaskError},
};

pub fn validate_research_project_title(title: &str) -> Result<String, ResearchProjectError> {
    let title = title.trim();
    if (research::MIN_RESEARCH_PROJECT_TITLE_LENGTH..=research::MAX_RESEARCH_PROJECT_TITLE_LENGTH)
        .contains(&title.chars().count())
    {
        Ok(title.to_owned())
    } else {
        Err(ResearchProjectError::TitleInvalid)
    }
}

pub fn validate_research_project_abstract(
    abstract_text: Option<&str>,
) -> Result<Option<String>, ResearchProjectError> {
    abstract_text
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.chars().count() <= research::MAX_RESEARCH_PROJECT_ABSTRACT_LENGTH {
                Ok(value.to_owned())
            } else {
                Err(ResearchProjectError::AbstractTooLong)
            }
        })
        .transpose()
}

pub fn validate_research_project_description(
    description: &str,
) -> Result<String, ResearchProjectError> {
    let description = description.trim();
    if description.chars().count() <= research::MAX_RESEARCH_PROJECT_DESCRIPTION_LENGTH {
        Ok(description.to_owned())
    } else {
        Err(ResearchProjectError::DescriptionTooLong)
    }
}

pub fn validate_research_project_objectives(
    objectives: &[String],
) -> Result<Vec<String>, ResearchProjectError> {
    validate_research_project_list(
        objectives,
        research::MAX_RESEARCH_PROJECT_OBJECTIVES,
        ResearchProjectError::TooManyObjectives,
        ResearchProjectError::InvalidObjective,
    )
}

pub fn validate_research_questions(
    questions: &[String],
) -> Result<Vec<String>, ResearchProjectError> {
    validate_research_project_list(
        questions,
        research::MAX_RESEARCH_PROJECT_QUESTIONS,
        ResearchProjectError::TooManyQuestions,
        ResearchProjectError::InvalidQuestion,
    )
}

pub fn validate_research_subject_ids(subject_ids: &[Uuid]) -> Result<(), ResearchProjectError> {
    if subject_ids.len() > research::MAX_RESEARCH_PROJECT_SUBJECTS {
        return Err(ResearchProjectError::TooManySubjects);
    }
    if subject_ids.iter().collect::<HashSet<_>>().len() != subject_ids.len() {
        return Err(ResearchProjectError::DuplicateSubject);
    }
    Ok(())
}

fn validate_research_project_list(
    values: &[String],
    maximum_count: usize,
    too_many_error: ResearchProjectError,
    invalid_value_error: ResearchProjectError,
) -> Result<Vec<String>, ResearchProjectError> {
    if values.len() > maximum_count {
        return Err(too_many_error);
    }
    values
        .iter()
        .map(|value| {
            let value = value.trim();
            if !value.is_empty()
                && value.chars().count() <= research::MAX_RESEARCH_PROJECT_ITEM_LENGTH
            {
                Ok(value.to_owned())
            } else {
                Err(invalid_value_error)
            }
        })
        .collect()
}

pub fn validate_research_task_title(title: &str) -> Result<String, ResearchTaskError> {
    let title = title.trim();
    if (research::MIN_RESEARCH_TASK_TITLE_LENGTH..=research::MAX_RESEARCH_TASK_TITLE_LENGTH)
        .contains(&title.chars().count())
    {
        Ok(title.to_owned())
    } else {
        Err(ResearchTaskError::TitleInvalid)
    }
}

pub fn validate_research_task_objective(objective: &str) -> Result<String, ResearchTaskError> {
    validate_research_task_text(
        objective,
        research::MAX_RESEARCH_TASK_OBJECTIVE_LENGTH,
        ResearchTaskError::ObjectiveInvalid,
    )
}

pub fn validate_research_task_scope(scope: &str) -> Result<String, ResearchTaskError> {
    validate_research_task_text(
        scope,
        research::MAX_RESEARCH_TASK_SCOPE_LENGTH,
        ResearchTaskError::ScopeInvalid,
    )
}

pub fn validate_research_task_data_sources(
    data_sources: &[String],
) -> Result<Vec<String>, ResearchTaskError> {
    validate_research_task_items(
        data_sources,
        ResearchTaskError::TooManyDataSources,
        ResearchTaskError::InvalidDataSource,
        research::MAX_RESEARCH_TASK_ITEM_LENGTH,
    )
}

pub fn validate_research_task_permitted_tools(
    permitted_tools: &[String],
) -> Result<Vec<String>, ResearchTaskError> {
    validate_research_task_items(
        permitted_tools,
        ResearchTaskError::TooManyPermittedTools,
        ResearchTaskError::InvalidPermittedTool,
        research::MAX_RESEARCH_TASK_TOOL_LENGTH,
    )
}

pub fn validate_research_task_deliverables(
    deliverables: &[String],
) -> Result<Vec<String>, ResearchTaskError> {
    if deliverables.is_empty() {
        return Err(ResearchTaskError::MissingDeliverables);
    }
    validate_research_task_items(
        deliverables,
        ResearchTaskError::TooManyDeliverables,
        ResearchTaskError::InvalidDeliverable,
        research::MAX_RESEARCH_TASK_ITEM_LENGTH,
    )
}

pub fn validate_research_task_runtime_limit(
    runtime_limit_seconds: i32,
) -> Result<(), ResearchTaskError> {
    if (1..=research::MAX_RESEARCH_TASK_RUNTIME_SECONDS).contains(&runtime_limit_seconds) {
        Ok(())
    } else {
        Err(ResearchTaskError::RuntimeLimitInvalid)
    }
}

pub fn validate_research_task_result_limit(
    result_limit_bytes: i32,
) -> Result<(), ResearchTaskError> {
    if (1..=research::MAX_RESEARCH_TASK_RESULT_BYTES as i32).contains(&result_limit_bytes) {
        Ok(())
    } else {
        Err(ResearchTaskError::ResultLimitInvalid)
    }
}

pub fn validate_research_task_result(
    result: &str,
    maximum_bytes: i32,
) -> Result<String, ResearchTaskError> {
    let result = result.trim();
    if !result.is_empty() && result.len() <= maximum_bytes as usize {
        Ok(result.to_owned())
    } else {
        Err(ResearchTaskError::ResultInvalid)
    }
}

pub fn validate_research_task_review_notes(
    review_notes: Option<&str>,
) -> Result<Option<String>, ResearchTaskError> {
    review_notes
        .map(str::trim)
        .filter(|notes| !notes.is_empty())
        .map(|notes| {
            if notes.len() <= research::MAX_RESEARCH_TASK_REVIEW_NOTES_BYTES {
                Ok(notes.to_owned())
            } else {
                Err(ResearchTaskError::ReviewNotesTooLong)
            }
        })
        .transpose()
}

fn validate_research_task_text(
    value: &str,
    maximum_length: usize,
    error: ResearchTaskError,
) -> Result<String, ResearchTaskError> {
    let value = value.trim();
    if !value.is_empty() && value.chars().count() <= maximum_length {
        Ok(value.to_owned())
    } else {
        Err(error)
    }
}

fn validate_research_task_items(
    values: &[String],
    too_many_error: ResearchTaskError,
    invalid_value_error: ResearchTaskError,
    maximum_length: usize,
) -> Result<Vec<String>, ResearchTaskError> {
    if values.len() > research::MAX_RESEARCH_TASK_ITEMS {
        return Err(too_many_error);
    }
    values
        .iter()
        .map(|value| {
            let value = value.trim();
            if !value.is_empty() && value.chars().count() <= maximum_length {
                Ok(value.to_owned())
            } else {
                Err(invalid_value_error)
            }
        })
        .collect()
}
