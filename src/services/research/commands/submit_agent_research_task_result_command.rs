use axum::http::Uri;
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

use crate::{
    constants::research::{
        MAX_RESEARCH_SOURCE_DOI_LENGTH, MAX_RESEARCH_SOURCE_TITLE_LENGTH,
        MAX_RESEARCH_SOURCE_URL_LENGTH, MAX_RESEARCH_TASK_SOURCE_REFERENCES,
    },
    dtos::agent_credential::AgentResearchTaskSourceRequest,
    enums::research::ResearchTaskError,
    errors::app_error::AppError,
    repositories::research_task_repository::{self, AgentTaskSource},
};

pub async fn submit_research_task_result_for_agent(
    pool: &PgPool,
    task_id: Uuid,
    agent_actor_id: Uuid,
    result: &str,
    sources: Vec<AgentResearchTaskSourceRequest>,
) -> Result<(), AppError> {
    let sources = validate_agent_research_sources(sources)?;
    research_task_repository::submit_research_task_result_for_agent(
        pool,
        task_id,
        agent_actor_id,
        result,
        &sources,
    )
    .await
}

fn validate_agent_research_sources(
    sources: Vec<AgentResearchTaskSourceRequest>,
) -> Result<Vec<AgentTaskSource>, AppError> {
    if sources.is_empty() || sources.len() > MAX_RESEARCH_TASK_SOURCE_REFERENCES {
        return Err(ResearchTaskError::SourceReferencesInvalid.into());
    }
    let mut validated_sources = Vec::with_capacity(sources.len());
    let mut source_urls = HashSet::with_capacity(sources.len());
    for source in sources {
        let title = source.title.trim();
        let url = source.url.trim();
        let parsed_url = url
            .parse::<Uri>()
            .map_err(|_| AppError::from(ResearchTaskError::SourceReferencesInvalid))?;
        let doi = source
            .doi
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        if title.is_empty()
            || title.chars().count() > MAX_RESEARCH_SOURCE_TITLE_LENGTH
            || url.chars().count() > MAX_RESEARCH_SOURCE_URL_LENGTH
            || parsed_url.scheme_str() != Some("https")
            || parsed_url.authority().is_none_or(|authority| {
                authority.host().is_empty() || authority.as_str().contains('@')
            })
            || doi
                .as_ref()
                .is_some_and(|value| value.len() > MAX_RESEARCH_SOURCE_DOI_LENGTH)
            || !source_urls.insert(url.to_owned())
        {
            return Err(ResearchTaskError::SourceReferencesInvalid.into());
        }
        validated_sources.push(AgentTaskSource {
            title: title.to_owned(),
            url: url.to_owned(),
            doi,
        });
    }
    Ok(validated_sources)
}
